use crate::models::PageInfo;

// ── Page title macros ────────────────────────────────────────────────────

#[macro_export]
macro_rules! page_title {
    ($i18n:expr, $key:ident) => {
        format!(
            "{} – {} | {}",
            $crate::i18n::t_string!($i18n, $key),
            $crate::i18n::t_string!($i18n, site_name),
            $crate::i18n::t_string!($i18n, site_slogan)
        )
    };
}

#[macro_export]
macro_rules! site_title {
    ($i18n:expr) => {
        format!(
            "{} | {}",
            $crate::i18n::t_string!($i18n, site_name),
            $crate::i18n::t_string!($i18n, site_slogan)
        )
    };
}

// ── Pagination (wasm + server) ────────────────────────────────────────────────

pub fn make_page_info(from: i64, ps: i64, total: u64) -> PageInfo {
    let tp = ((total as f64 / ps as f64).ceil() as u32).max(1);
    PageInfo {
        current_page: from as u32,
        total_pages: tp,
        total_count: total,
        first_cursor: String::new(),
        last_cursor: String::new(),
        has_previous: from > 1,
        has_next: (from as u32) < tp,
    }
}

// ── RecordId helpers (client + server) ────────────────────────────────────────

/// Extract the bare key from a "table:id" string, for URL path / query generation.
/// `"footballs:abc123"` → `"abc123"`
/// `"abc123"`           → `"abc123"` (passthrough for bare keys)
pub fn record_key(full: &str) -> &str {
    full.rsplit_once(':').map(|(_, k)| k).unwrap_or(full)
}

// ── SurrealDB helpers (server only) ───────────────────────────────────────────

#[cfg(feature = "ssr")]
use surrealdb::types::{RecordId, RecordIdKey};

#[cfg(feature = "ssr")]
/// `RecordIdKey` → plain string (internal helper)
fn key_str(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        _ => format!("{key:?}"),
    }
}

#[cfg(feature = "ssr")]
/// `RecordId` → `"table:id"` string (use this instead of Display, which RecordId lacks)
pub fn rid_str(r: &RecordId) -> String {
    format!("{}:{}", r.table, key_str(&r.key))
}

#[cfg(feature = "ssr")]
/// Parse a string into `RecordId`.
/// If `input` contains `:` it is parsed as `"table:key"`; otherwise `default_table` is prepended.
pub fn into_rid(input: &str, default_table: &str) -> RecordId {
    if let Some((tbl, key)) = input.split_once(':') {
        RecordId::new(tbl, key)
    } else {
        RecordId::new(default_table, input)
    }
}

// ── Datetime helpers (server only) ────────────────────────────────────────────

/// Format a SurrealDB Datetime as "%Y-%m-%d" in UTC+8.
#[cfg(feature = "ssr")]
pub fn ymd8(dt: &surrealdb::types::Datetime) -> String {
    use chrono::FixedOffset;
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%Y-%m-%d").to_string()
}

/// Format a SurrealDB Datetime as "%Y-%m-%d %H:%M:%S%:z" in UTC+8.
#[cfg(feature = "ssr")]
pub fn ymdhmsz8(dt: &surrealdb::types::Datetime) -> String {
    use chrono::FixedOffset;
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8)
        .format("%Y-%m-%d %H:%M:%S%:z")
        .to_string()
}
