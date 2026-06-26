//! Typed errors for Hippocore DB.
//!
//! Every fallible public operation returns [`Result`]; normal failures are
//! represented as a [`HippocoreError`] variant rather than a panic.

use thiserror::Error;

/// Convenience result type used throughout the crate.
pub type Result<T> = std::result::Result<T, HippocoreError>;

/// All errors the database can surface.
#[derive(Debug, Error)]
pub enum HippocoreError {
    /// An underlying I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON (de)serialization of a record or snapshot failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// A model failed validation (empty required field, bad value, ...).
    #[error("validation error: {0}")]
    Validation(String),

    /// The referenced tenant does not exist.
    #[error("unknown tenant: {0:?}")]
    UnknownTenant(String),

    /// The referenced collection does not exist for the tenant.
    #[error("unknown collection {collection:?} for tenant {tenant:?}")]
    UnknownCollection {
        /// Tenant the collection was looked up under.
        tenant: String,
        /// Collection name that was not found.
        collection: String,
    },

    /// An entity with the same identity already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// A requested entity was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// A vector operation received an empty or mismatched embedding.
    #[error("invalid embedding: {0}")]
    InvalidEmbedding(String),

    /// On-disk data could not be interpreted; recovery stopped safely.
    #[error("data corruption: {0}")]
    Corruption(String),
}

impl HippocoreError {
    /// Helper to build a [`HippocoreError::Validation`] from any message.
    pub fn validation(msg: impl Into<String>) -> Self {
        HippocoreError::Validation(msg.into())
    }
}
