// ── CSS class constants (shared) ────────────────────────────────────────────
pub const NO_UNDERLINE: &str = "no-underline";
pub const HOVER_NO_UNDERLINE: &str = "hover:no-underline";
pub const HOVER_UNDERLINE: &str = "hover:underline";
pub const BADGE_BLUE: &str = "badge-blue";
pub const BADGE_BLUE_NO_UL: &str = "badge-blue no-underline";
pub const BADGE_GRAY: &str = "badge-gray";
pub const BADGE_GRAY_NO_UL: &str = "badge-gray no-underline";
pub const BADGE_GREEN: &str = "badge-green";
pub const BADGE_RED: &str = "badge-red";
pub const CAT_ITEM: &str = "px-2 py-1 text-sm no-underline";
pub const ITALIC: &str = "italic";
pub const ITALIC_XS: &str = "text-xs text-gray-400 italic";

// ── Config (SSR only) ──────────────────────────────────────────────────────
#[cfg(feature = "ssr")]
use std::sync::LazyLock;

#[cfg(feature = "ssr")]
pub struct Config {
    pub domain: String,
    pub site_key: String,
    pub claim_exp: usize,
    pub page_size: i64,
    pub db_url: String,
    pub db_ns: String,
    pub db_name: String,
    pub db_user: String,
    pub db_pass: String,
    pub email_smtp: String,
    pub email_from: String,
    pub email_username: String,
    pub email_password: String,
}

#[cfg(feature = "ssr")]
static CFG: LazyLock<Config> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    Config {
        domain: env("DOMAIN"),
        site_key: env("SITE_KEY"),
        claim_exp: now() + parse::<usize>("CLAIM_EXP"),
        page_size: parse::<i64>("PAGE_SIZE"),
        db_url: env("DB_URL"),
        db_ns: env("DB_NS"),
        db_name: env("DB_NAME"),
        db_user: env("DB_USER"),
        db_pass: env("DB_PASS"),
        email_smtp: env("EMAIL_SMTP"),
        email_from: env("EMAIL_FROM"),
        email_username: env("EMAIL_USERNAME"),
        email_password: env("EMAIL_PASSWORD"),
    }
});

#[cfg(feature = "ssr")]
pub fn config() -> &'static Config {
    &CFG
}

// ── helpers (SSR only) ──────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
}

#[cfg(feature = "ssr")]
fn parse<T: std::str::FromStr>(key: &str) -> T
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    std::env::var(key)
        .unwrap_or_else(|_| panic!("{key} must be set"))
        .parse()
        .unwrap_or_else(|e| panic!("{key} must be a valid integer: {e}"))
}

#[cfg(feature = "ssr")]
fn now() -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}
