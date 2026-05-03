use base64::{engine::general_purpose::STANDARD, Engine};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CRED_LEN: usize = ring::digest::SHA256_OUTPUT_LEN;
const ITERATIONS: u32 = 100_000;

/// Application-level salt prefix (16 fixed bytes) combined with the username.
const SALT_PREFIX: [u8; 16] = [
    0x7f, 0x3a, 0x9c, 0x42, 0x1e, 0x88, 0xd5, 0x6b, 0x4f, 0x2c, 0xe7, 0x93, 0x5a, 0x0d, 0xb8, 0x61,
];

fn make_salt(username: &str) -> Vec<u8> {
    let mut s = SALT_PREFIX.to_vec();
    s.extend_from_slice(username.as_bytes());
    s
}

pub fn hash_credential(username: &str, password: &str) -> String {
    let salt = make_salt(username);
    let mut cred = vec![0u8; CRED_LEN];
    pbkdf2::derive(
        PBKDF2_ALG,
        NonZeroU32::new(ITERATIONS).unwrap(),
        &salt,
        password.as_bytes(),
        &mut cred,
    );
    STANDARD.encode(&cred)
}

pub fn verify_credential(username: &str, password: &str, stored: &str) -> bool {
    let Ok(stored_bytes) = STANDARD.decode(stored) else {
        return false;
    };
    let salt = make_salt(username);
    pbkdf2::verify(
        PBKDF2_ALG,
        NonZeroU32::new(ITERATIONS).unwrap(),
        &salt,
        password.as_bytes(),
        &stored_bytes,
    )
    .is_ok()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub username: String,
    pub exp: usize,
}

fn site_key() -> String {
    std::env::var("SITE_KEY").expect("SITE_KEY must be set")
}

fn site_kid() -> String {
    std::env::var("SITE_KID").unwrap_or_else(|_| "football-site".to_string())
}

fn claim_exp() -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    std::env::var("CLAIM_EXP")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .map(|secs| now + secs)
        .unwrap_or(now + 30 * 24 * 3600)
}

pub fn encode_jwt(email: &str, username: &str) -> Result<String, String> {
    let claims = Claims {
        email: email.to_string(),
        username: username.to_string(),
        exp: claim_exp(),
    };
    let mut header = Header::new(Algorithm::HS512);
    header.kid = Some(site_kid());
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(site_key().as_bytes()),
    )
    .map_err(|e| e.to_string())
}

pub fn decode_jwt(token: &str) -> Result<Claims, String> {
    let mut val = Validation::new(Algorithm::HS512);
    val.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(site_key().as_bytes()),
        &val,
    )
    .map(|td| td.claims)
    .map_err(|e| e.to_string())
}

/// Extract a named cookie value from a raw `Cookie` header string.
pub fn get_cookie_value(header: &str, name: &str) -> Option<String> {
    for part in header.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(name) {
            if let Some(val) = rest.strip_prefix('=') {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Build a `Set-Cookie` header value that sets a cookie.
pub fn make_set_cookie(name: &str, value: &str, max_age: i64) -> String {
    format!("{name}={value}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age}")
}

/// Build a `Set-Cookie` header value that clears a cookie.
pub fn make_clear_cookie(name: &str) -> String {
    format!("{name}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
}
