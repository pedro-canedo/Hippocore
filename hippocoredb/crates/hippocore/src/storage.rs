//! Durable persistence: a JSON-lines write-ahead log plus an atomic snapshot.
//!
//! ## Files (inside `data_dir`)
//!
//! - `wal.log`   — one operation per line: `<crc32-hex>\t<json>`, appended in
//!   order. The CRC32 covers the JSON payload so corruption is detectable.
//! - `snapshot.json` — a compacted [`State`] written atomically.
//! - `meta.json` — format metadata.
//!
//! ## Recovery
//!
//! On open we load `snapshot.json` (if present) as the base state, then replay
//! every line of `wal.log` on top. Replay stops safely (keeping everything read
//! so far) at the first sign of trouble — a truncated trailing line, a checksum
//! mismatch, or an unparseable payload — so a crash mid-write or on-disk bit-rot
//! never panics and never silently loads corrupt data.
//!
//! Legacy WAL lines without a checksum prefix (format v1) are still accepted, so
//! existing data directories keep working.
//!
//! ## Compaction
//!
//! [`Storage::compact`] writes the current state to a fresh snapshot atomically
//! (temp file + `fsync` + rename) and then truncates the WAL.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::{HippocoreError, Result};
use crate::model::{Chunk, Collection, Document, FileObject, Memory, Record, Tenant};

const WAL_FILE: &str = "wal.log";
const SNAPSHOT_FILE: &str = "snapshot.json";
const META_FILE: &str = "meta.json";
const FORMAT_VERSION: u32 = 2;

/// CRC32 (IEEE 802.3) of `data`. Dependency-free; used for WAL line integrity,
/// not for security.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// A single mutation, as recorded in the WAL and replayed on recovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Operation {
    /// Register a tenant.
    CreateTenant(Tenant),
    /// Register a collection.
    CreateCollection(Collection),
    /// Store (or overwrite) a document together with its derived chunks.
    PutDocument {
        /// The stored document.
        document: Document,
        /// Chunks derived from the document text.
        chunks: Vec<Chunk>,
    },
    /// Store (or overwrite) a memory.
    PutMemory(Memory),
    /// Store (or overwrite) a structured record.
    PutRecord(Record),
    /// Store (or overwrite) a file object and its derived document/chunks.
    PutFile {
        /// Imported file metadata.
        file: FileObject,
        /// Derived document containing extracted text.
        document: Document,
        /// Chunks derived from the extracted text.
        chunks: Vec<Chunk>,
    },
    /// Remove a memory by identity (tombstone). No-op if absent.
    DeleteMemory {
        /// Owning tenant.
        tenant_id: String,
        /// Owning collection.
        collection: String,
        /// Memory id.
        id: String,
    },
    /// Remove a document and all its chunks by identity (tombstone). No-op if
    /// absent.
    DeleteDocument {
        /// Owning tenant.
        tenant_id: String,
        /// Owning collection.
        collection: String,
        /// Document id.
        id: String,
    },
    /// Remove a structured record by identity (tombstone). No-op if absent.
    DeleteRecord {
        /// Owning tenant.
        tenant_id: String,
        /// Owning collection.
        collection: String,
        /// Logical table namespace.
        table: String,
        /// Record id.
        id: String,
    },
    /// Remove an imported file and its derived document/chunks by identity.
    DeleteFile {
        /// Owning tenant.
        tenant_id: String,
        /// Owning collection.
        collection: String,
        /// File id.
        id: String,
    },
}

/// The fully materialized database state. Indexes are rebuilt from this.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    /// Tenants by id.
    pub tenants: Vec<Tenant>,
    /// Collections (tenant_id, name unique).
    pub collections: Vec<Collection>,
    /// Documents.
    pub documents: Vec<Document>,
    /// Chunks (searchable units of documents).
    pub chunks: Vec<Chunk>,
    /// Memories (searchable long-term items).
    #[serde(default)]
    pub memories: Vec<Memory>,
    /// Structured records.
    #[serde(default)]
    pub records: Vec<Record>,
    /// Imported file metadata.
    #[serde(default)]
    pub files: Vec<FileObject>,
}

impl State {
    /// Apply an operation to the state, replacing any existing entity that
    /// shares the same identity (last write wins).
    pub fn apply(&mut self, op: Operation) {
        match op {
            Operation::CreateTenant(t) => {
                self.tenants.retain(|x| x.id != t.id);
                self.tenants.push(t);
            }
            Operation::CreateCollection(c) => {
                self.collections
                    .retain(|x| !(x.tenant_id == c.tenant_id && x.name == c.name));
                self.collections.push(c);
            }
            Operation::PutDocument { document, chunks } => {
                self.documents.retain(|d| {
                    !(d.tenant_id == document.tenant_id
                        && d.collection == document.collection
                        && d.id == document.id)
                });
                self.chunks.retain(|ch| {
                    !(ch.tenant_id == document.tenant_id
                        && ch.collection == document.collection
                        && ch.document_id == document.id)
                });
                self.documents.push(document);
                self.chunks.extend(chunks);
            }
            Operation::PutMemory(m) => {
                self.memories.retain(|x| {
                    !(x.tenant_id == m.tenant_id && x.collection == m.collection && x.id == m.id)
                });
                self.memories.push(m);
            }
            Operation::PutRecord(r) => {
                self.records.retain(|x| {
                    !(x.tenant_id == r.tenant_id
                        && x.collection == r.collection
                        && x.table == r.table
                        && x.id == r.id)
                });
                self.records.push(r);
            }
            Operation::PutFile {
                file,
                document,
                chunks,
            } => {
                self.files.retain(|x| {
                    !(x.tenant_id == file.tenant_id
                        && x.collection == file.collection
                        && x.id == file.id)
                });
                self.documents.retain(|d| {
                    !(d.tenant_id == document.tenant_id
                        && d.collection == document.collection
                        && d.id == document.id)
                });
                self.chunks.retain(|ch| {
                    !(ch.tenant_id == document.tenant_id
                        && ch.collection == document.collection
                        && ch.document_id == document.id)
                });
                self.files.push(file);
                self.documents.push(document);
                self.chunks.extend(chunks);
            }
            Operation::DeleteMemory {
                tenant_id,
                collection,
                id,
            } => {
                self.memories.retain(|x| {
                    !(x.tenant_id == tenant_id && x.collection == collection && x.id == id)
                });
            }
            Operation::DeleteDocument {
                tenant_id,
                collection,
                id,
            } => {
                self.documents.retain(|d| {
                    !(d.tenant_id == tenant_id && d.collection == collection && d.id == id)
                });
                self.chunks.retain(|ch| {
                    !(ch.tenant_id == tenant_id
                        && ch.collection == collection
                        && ch.document_id == id)
                });
            }
            Operation::DeleteRecord {
                tenant_id,
                collection,
                table,
                id,
            } => {
                self.records.retain(|x| {
                    !(x.tenant_id == tenant_id
                        && x.collection == collection
                        && x.table == table
                        && x.id == id)
                });
            }
            Operation::DeleteFile {
                tenant_id,
                collection,
                id,
            } => {
                if let Some(file) = self
                    .files
                    .iter()
                    .find(|x| x.tenant_id == tenant_id && x.collection == collection && x.id == id)
                    .cloned()
                {
                    self.documents.retain(|d| {
                        !(d.tenant_id == tenant_id
                            && d.collection == collection
                            && d.id == file.document_id)
                    });
                    self.chunks.retain(|ch| {
                        !(ch.tenant_id == tenant_id
                            && ch.collection == collection
                            && ch.document_id == file.document_id)
                    });
                }
                self.files.retain(|x| {
                    !(x.tenant_id == tenant_id && x.collection == collection && x.id == id)
                });
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Meta {
    format_version: u32,
}

/// Owns the on-disk files and the append handle to the WAL.
pub struct Storage {
    data_dir: PathBuf,
    wal: File,
    sync_writes: bool,
    /// Number of operations appended/replayed since the last compaction.
    pub wal_len: usize,
    /// Size of the live WAL in bytes since the last compaction.
    pub wal_bytes: u64,
}

impl Storage {
    /// Open (creating if needed) the storage rooted at `data_dir`, returning the
    /// recovered [`State`].
    pub fn open(data_dir: &Path, sync_writes: bool) -> Result<(Self, State)> {
        std::fs::create_dir_all(data_dir)?;
        Self::write_meta_if_absent(data_dir)?;

        let mut state = Self::load_snapshot(data_dir)?;
        let wal_path = data_dir.join(WAL_FILE);
        let replayed = Self::replay_wal(&wal_path, &mut state)?;

        let wal = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&wal_path)?;
        let wal_bytes = wal.metadata()?.len();

        Ok((
            Self {
                data_dir: data_dir.to_path_buf(),
                wal,
                sync_writes,
                wal_len: replayed,
                wal_bytes,
            },
            state,
        ))
    }

    fn write_meta_if_absent(dir: &Path) -> Result<()> {
        let path = dir.join(META_FILE);
        if !path.exists() {
            let meta = Meta {
                format_version: FORMAT_VERSION,
            };
            atomic_write(&path, serde_json::to_vec_pretty(&meta)?.as_slice())?;
        }
        Ok(())
    }

    fn load_snapshot(dir: &Path) -> Result<State> {
        let path = dir.join(SNAPSHOT_FILE);
        if !path.exists() {
            return Ok(State::default());
        }
        let bytes = std::fs::read(&path)?;
        let state: State = serde_json::from_slice(&bytes)
            .map_err(|e| HippocoreError::Corruption(format!("snapshot.json: {e}")))?;
        Ok(state)
    }

    /// Replay each WAL line into `state`. Replay stops safely at the first
    /// truncated line, checksum mismatch, or unparseable payload, keeping
    /// everything applied so far. This prevents both panics and the silent
    /// loading of corrupt data.
    fn replay_wal(path: &Path, state: &mut State) -> Result<usize> {
        if !path.exists() {
            return Ok(0);
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut applied = 0;
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break, // truncated trailing line: stop safely
            };
            if line.trim().is_empty() {
                continue;
            }
            match decode_wal_line(&line) {
                // Clean, checksum-verified (or legacy) operation.
                Ok(Some(op)) => {
                    state.apply(op);
                    applied += 1;
                }
                // Unparseable trailing line (e.g. half-written): stop safely.
                Ok(None) => break,
                // Checksum mismatch: on-disk corruption. Do not load it or
                // anything after it.
                Err(_) => break,
            }
        }
        Ok(applied)
    }

    /// Append an operation durably as `<crc32-hex>\t<json>\n`.
    pub fn append(&mut self, op: &Operation) -> Result<()> {
        let json = serde_json::to_vec(op)?;
        let crc = crc32(&json);
        let mut line = format!("{crc:08x}\t").into_bytes();
        line.extend_from_slice(&json);
        line.push(b'\n');

        self.wal.write_all(&line)?;
        self.wal.flush()?;
        if self.sync_writes {
            self.wal.sync_all()?;
        }
        self.wal_len += 1;
        self.wal_bytes += line.len() as u64;
        Ok(())
    }

    /// Write a fresh snapshot of `state` atomically and truncate the WAL.
    pub fn compact(&mut self, state: &State) -> Result<()> {
        let snapshot_path = self.data_dir.join(SNAPSHOT_FILE);
        let bytes = serde_json::to_vec(state)?;
        atomic_write(&snapshot_path, &bytes)?;

        // Truncate the WAL now that its contents are folded into the snapshot.
        // The handle stays in append mode; subsequent writes land at offset 0.
        self.wal.set_len(0)?;
        self.wal.sync_all()?;
        self.wal_len = 0;
        self.wal_bytes = 0;
        Ok(())
    }

    /// Flush and `fsync` the WAL.
    pub fn sync(&mut self) -> Result<()> {
        self.wal.flush()?;
        self.wal.sync_all()?;
        Ok(())
    }

    /// Total size of WAL + snapshot in bytes.
    pub fn disk_bytes(&self) -> Result<u64> {
        let mut total = 0;
        for name in [WAL_FILE, SNAPSHOT_FILE] {
            let p = self.data_dir.join(name);
            if p.exists() {
                total += std::fs::metadata(p)?.len();
            }
        }
        Ok(total)
    }

    /// Path to the data directory.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

/// Decode one WAL line.
///
/// - `Ok(Some(op))` — a checksum-verified (or legacy, unchecksummed) operation.
/// - `Ok(None)` — the payload could not be parsed (e.g. a half-written trailing
///   line); the caller should stop replay here.
/// - `Err(..)` — a checksum mismatch: on-disk corruption that must not be loaded.
fn decode_wal_line(line: &str) -> Result<Option<Operation>> {
    match line.split_once('\t') {
        Some((prefix, payload)) if is_crc_prefix(prefix) => {
            let expected = u32::from_str_radix(prefix, 16)
                .map_err(|e| HippocoreError::Corruption(format!("bad crc prefix: {e}")))?;
            if crc32(payload.as_bytes()) != expected {
                return Err(HippocoreError::Corruption(
                    "WAL line checksum mismatch".into(),
                ));
            }
            Ok(serde_json::from_str::<Operation>(payload).ok())
        }
        // No checksum prefix: a legacy (format v1) line is the whole JSON.
        _ => Ok(serde_json::from_str::<Operation>(line).ok()),
    }
}

fn is_crc_prefix(prefix: &str) -> bool {
    prefix.len() == 8 && prefix.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Write `bytes` to `path` atomically: write to a temp file, `fsync`, rename.
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.flush()?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_op() -> Operation {
        Operation::CreateTenant(Tenant {
            id: "t".into(),
            name: "T".into(),
            created_at: 0,
        })
    }

    #[test]
    fn crc32_matches_known_vector() {
        // CRC32 of "123456789" is 0xCBF43926 (standard test vector).
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn checksummed_line_roundtrips() {
        let op = sample_op();
        let json = serde_json::to_string(&op).unwrap();
        let line = format!("{:08x}\t{json}", crc32(json.as_bytes()));
        assert_eq!(decode_wal_line(&line).unwrap(), Some(op));
    }

    #[test]
    fn legacy_line_without_checksum_is_accepted() {
        let op = sample_op();
        let json = serde_json::to_string(&op).unwrap(); // no crc prefix
        assert_eq!(decode_wal_line(&json).unwrap(), Some(op));
    }

    #[test]
    fn checksum_mismatch_is_detected() {
        let op = sample_op();
        let json = serde_json::to_string(&op).unwrap();
        // Use a wrong-but-well-formed crc prefix.
        let line = format!("00000000\t{json}");
        assert!(decode_wal_line(&line).is_err());
    }

    #[test]
    fn unparseable_payload_signals_stop() {
        let line = format!("{:08x}\t{{ not json", crc32(b"{ not json"));
        assert_eq!(decode_wal_line(&line).unwrap(), None);
    }
}
