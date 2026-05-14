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

// ── SurrealDB helpers (server only) ───────────────────────────────────────────

#[cfg(feature = "ssr")]
use surrealdb::types::{RecordId, RecordIdKey};

#[cfg(feature = "ssr")]
pub fn id_only(r: &RecordId) -> String {
    match &r.key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        _ => format!("{r:?}"),
    }
}

#[cfg(feature = "ssr")]
pub fn record_id(table: &str, id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("{}:{}", table, id)
    }
}

#[cfg(feature = "ssr")]
pub fn rid_str(r: &RecordId) -> String {
    format!("{}:{}", r.table, id_only(r))
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
