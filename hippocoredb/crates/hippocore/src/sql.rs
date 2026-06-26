//! Tiny SQL-like command layer for database-style record queries.
//!
//! This module is intentionally narrow. It is a typed command parser/executor
//! boundary, not a general SQL engine.

use serde::Serialize;

use crate::errors::{HippocoreError, Result};
use crate::model::{Metadata, Record};

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 500;

/// A parsed MVP SQL-like command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlCommand {
    /// Read records from a logical table namespace.
    SelectRecords(RestrictedRecordQuery),
}

/// Result returned by [`crate::Hippocore::execute_sql`].
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SqlResult {
    /// Command kind, currently always `"select"`.
    pub command: String,
    /// Number of rows returned after filtering and limit.
    pub row_count: usize,
    /// Returned records.
    pub rows: Vec<Record>,
}

impl SqlResult {
    pub(crate) fn records(rows: Vec<Record>) -> Self {
        Self {
            command: "select".to_string(),
            row_count: rows.len(),
            rows,
        }
    }
}

/// A restricted, read-only query over stored structured records.
///
/// Supported syntax:
/// `select * from <table> [where field = 'value' [and ...]] [limit n]`.
///
/// `select * from records ...` remains compatible with the older API and does
/// not imply a table filter by itself. Any other `FROM` target becomes the
/// `Record.table` filter. Bare fields map to top-level payload keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestrictedRecordQuery {
    /// Optional collection equality filter.
    pub collection: Option<String>,
    /// Optional table equality filter.
    pub table: Option<String>,
    /// Optional record id equality filter.
    pub id: Option<String>,
    /// Exact metadata filters.
    pub metadata: Metadata,
    /// Exact top-level payload filters, compared as scalar strings.
    pub payload: Metadata,
    /// Maximum number of rows returned.
    pub limit: usize,
}

impl Default for RestrictedRecordQuery {
    fn default() -> Self {
        Self {
            collection: None,
            table: None,
            id: None,
            metadata: Metadata::new(),
            payload: Metadata::new(),
            limit: DEFAULT_LIMIT,
        }
    }
}

/// Parse the public SQL command syntax.
pub fn parse_sql(sql: &str) -> Result<SqlCommand> {
    Ok(SqlCommand::SelectRecords(parse_select_records(sql)?))
}

/// Parse the legacy restricted records query syntax.
pub fn parse_restricted_record_query(sql: &str) -> Result<RestrictedRecordQuery> {
    let trimmed = clean_sql(sql);
    let lower = trimmed.to_ascii_lowercase();
    let prefix = "select * from records";
    if !lower.starts_with(prefix) {
        return Err(HippocoreError::validation(
            "restricted query must start with: select * from records",
        ));
    }
    parse_select_records(sql)
}

pub(crate) fn record_matches_restricted_query(
    record: &Record,
    query: &RestrictedRecordQuery,
) -> bool {
    if query.id.as_ref().is_some_and(|id| record.id != *id) {
        return false;
    }
    if query
        .metadata
        .iter()
        .any(|(key, value)| record.metadata.get(key) != Some(value))
    {
        return false;
    }
    query.payload.iter().all(|(key, value)| {
        record
            .payload
            .get(key)
            .and_then(json_scalar_as_string)
            .as_ref()
            == Some(value)
    })
}

fn parse_select_records(sql: &str) -> Result<RestrictedRecordQuery> {
    let trimmed = clean_sql(sql);
    let lower = trimmed.to_ascii_lowercase();
    let prefix = "select * from ";
    if !lower.starts_with(prefix) {
        return Err(HippocoreError::validation(
            "SQL command must start with: select * from <table>",
        ));
    }

    let mut query = RestrictedRecordQuery::default();
    let after_from = trimmed[prefix.len()..].trim();
    let (table, rest) = parse_identifier_token(after_from)?;
    if !table.eq_ignore_ascii_case("records") {
        query.table = Some(table.to_string());
    }

    let mut rest = rest.trim();
    if starts_with_keyword(rest, "where") {
        rest = rest[5..].trim();
        let (where_clause, after_where) =
            if let Some(idx) = find_keyword_outside_quotes(rest, "limit") {
                (&rest[..idx], rest[idx..].trim())
            } else {
                (rest, "")
            };
        parse_conditions(where_clause, &mut query)?;
        rest = after_where;
    }

    if starts_with_keyword(rest, "limit") {
        rest = rest[5..].trim();
        let n = rest
            .parse::<usize>()
            .map_err(|_| HippocoreError::validation("limit must be a positive integer"))?;
        if n == 0 {
            return Err(HippocoreError::validation(
                "limit must be greater than zero",
            ));
        }
        query.limit = n.min(MAX_LIMIT);
        rest = "";
    }

    if !rest.trim().is_empty() {
        return Err(HippocoreError::validation(
            "SQL SELECT only supports optional WHERE and LIMIT clauses",
        ));
    }

    Ok(query)
}

fn parse_identifier_token(input: &str) -> Result<(&str, &str)> {
    let input = input.trim();
    if input.is_empty() {
        return Err(HippocoreError::validation("missing table name"));
    }
    let end = input.find(char::is_whitespace).unwrap_or(input.len());
    let ident = &input[..end];
    if !is_identifier(ident) {
        return Err(HippocoreError::validation(format!(
            "invalid table name: {ident}"
        )));
    }
    Ok((ident, &input[end..]))
}

fn parse_conditions(where_clause: &str, query: &mut RestrictedRecordQuery) -> Result<()> {
    for condition in split_conditions(where_clause)? {
        let idx = find_char_outside_quotes(condition, '=')
            .ok_or_else(|| HippocoreError::validation("where conditions must use equality"))?;
        let field = condition[..idx].trim();
        let value = unquote_value(condition[idx + 1..].trim())?;
        add_condition(query, field, value)?;
    }
    Ok(())
}

fn add_condition(query: &mut RestrictedRecordQuery, field: &str, value: String) -> Result<()> {
    if field.is_empty() {
        return Err(HippocoreError::validation("where field must not be empty"));
    }
    let field_lower = field.to_ascii_lowercase();
    match field_lower.as_str() {
        "collection" => query.collection = Some(value),
        "table" => query.table = Some(value),
        "id" => query.id = Some(value),
        _ if field_lower.starts_with("metadata.") => {
            let key = field[9..].trim();
            if key.is_empty() {
                return Err(HippocoreError::validation("metadata key must not be empty"));
            }
            query.metadata.insert(key.to_string(), value);
        }
        _ if field_lower.starts_with("payload.") => {
            let key = field[8..].trim();
            if key.is_empty() {
                return Err(HippocoreError::validation("payload key must not be empty"));
            }
            query.payload.insert(key.to_string(), value);
        }
        _ => {
            if !is_identifier(field) {
                return Err(HippocoreError::validation(format!(
                    "unsupported SQL field: {field}"
                )));
            }
            query.payload.insert(field.to_string(), value);
        }
    }
    Ok(())
}

fn split_conditions(where_clause: &str) -> Result<Vec<&str>> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let bytes = where_clause.as_bytes();
    let mut quote = None;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' | b'"' => {
                if quote == Some(bytes[i]) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(bytes[i]);
                }
                i += 1;
            }
            _ if quote.is_none() && keyword_at(where_clause, i, "and") => {
                out.push(where_clause[start..i].trim());
                i += 3;
                start = i;
            }
            _ => i += 1,
        }
    }
    if quote.is_some() {
        return Err(HippocoreError::validation("unterminated quoted string"));
    }
    out.push(where_clause[start..].trim());
    Ok(out.into_iter().filter(|s| !s.is_empty()).collect())
}

fn unquote_value(value: &str) -> Result<String> {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let quoted = (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
            || (bytes[0] == b'"' && bytes[value.len() - 1] == b'"');
        if quoted {
            return Ok(value[1..value.len() - 1].to_string());
        }
    }
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(value.to_string())
    } else {
        Err(HippocoreError::validation(
            "query values must be quoted or simple identifiers",
        ))
    }
}

fn clean_sql(sql: &str) -> &str {
    sql.trim().trim_end_matches(';').trim()
}

fn starts_with_keyword(input: &str, keyword: &str) -> bool {
    keyword_at(input, 0, keyword)
}

fn find_keyword_outside_quotes(input: &str, keyword: &str) -> Option<usize> {
    let mut quote = None;
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' | b'"' => {
                if quote == Some(bytes[i]) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(bytes[i]);
                }
                i += 1;
            }
            _ if quote.is_none() && keyword_at(input, i, keyword) => return Some(i),
            _ => i += 1,
        }
    }
    None
}

fn find_char_outside_quotes(input: &str, ch: char) -> Option<usize> {
    let mut quote = None;
    for (idx, c) in input.char_indices() {
        match c {
            '\'' | '"' => {
                if quote == Some(c) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(c);
                }
            }
            _ if quote.is_none() && c == ch => return Some(idx),
            _ => {}
        }
    }
    None
}

fn keyword_at(input: &str, idx: usize, keyword: &str) -> bool {
    if idx > input.len() || !input[idx..].to_ascii_lowercase().starts_with(keyword) {
        return false;
    }
    let before_ok = idx == 0
        || input[..idx]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
    let after_idx = idx + keyword.len();
    let after_ok = after_idx == input.len()
        || input[after_idx..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace);
    before_ok && after_ok
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn json_scalar_as_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => Some("null".to_string()),
        serde_json::Value::Bool(v) => Some(v.to_string()),
        serde_json::Value::Number(v) => Some(v.to_string()),
        serde_json::Value::String(v) => Some(v.clone()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_table_and_bare_payload_fields() {
        let cmd =
            parse_sql("SELECT * FROM systems WHERE engine = 'postgresql' AND env = prod LIMIT 5")
                .unwrap();
        let SqlCommand::SelectRecords(q) = cmd;
        assert_eq!(q.table.as_deref(), Some("systems"));
        assert_eq!(
            q.payload.get("engine").map(String::as_str),
            Some("postgresql")
        );
        assert_eq!(q.payload.get("env").map(String::as_str), Some("prod"));
        assert_eq!(q.limit, 5);
    }

    #[test]
    fn preserves_legacy_records_table_semantics() {
        let q = parse_restricted_record_query(
            "select * from records where table = 'systems' and metadata.tier = 'database'",
        )
        .unwrap();
        assert_eq!(q.table.as_deref(), Some("systems"));
        assert_eq!(q.metadata.get("tier").map(String::as_str), Some("database"));
    }

    #[test]
    fn does_not_split_keywords_inside_quotes() {
        let q = parse_sql("select * from notes where title = 'research and development' limit 1")
            .unwrap();
        let SqlCommand::SelectRecords(q) = q;
        assert_eq!(
            q.payload.get("title").map(String::as_str),
            Some("research and development")
        );
    }

    #[test]
    fn rejects_unsupported_command() {
        let err = parse_sql("delete from records where id = 'x'").unwrap_err();
        assert!(err.to_string().contains("select * from"));
    }
}
