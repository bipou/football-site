use crate::utils::common;
use serde::Deserialize;
use surrealdb::types::{Datetime, RecordId, SurrealValue};

use crate::models::{AuthUser, User, UserSummary, UsersResult};
use crate::server::{auth as auth_mod, db::get_db, topic_db};
use crate::utils::common::{record_key, rid_str};
use crate::utils::constant;

// ── helpers ──────────────────────────────────────────────────────────────

fn render_md(md: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    let mut out = String::new();
    html::push_html(&mut out, Parser::new_ext(md, opts));
    out
}

// ── document structs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, SurrealValue)]
pub struct UserDoc {
    id: RecordId,
    username: String,
    email: String,
    cred: String,
    #[serde(default)]
    introduction: String,
    created_at: Datetime,
    updated_at: Datetime,
    status: i8,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct CountResult {
    count: u64,
}

// ── RegisterData ─────────────────────────────────────────────────────────

pub struct RegisterData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub introduction: String,
    pub topics: String,
}

// ── public functions ─────────────────────────────────────────────────────

/// Paginated list of active users (status >= 1), newest first.
pub async fn get_users(from: i64) -> Result<UsersResult, String> {
    let ps = constant::config().page_size;
    let skip = ((from - 1) * ps).max(0);

    // total count
    let mut resp = get_db()
        .query("SELECT count() FROM users WHERE status >= 1 GROUP ALL")
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = resp.take(0).map_err(|e| e.to_string())?;
    let total = counts.first().map(|c| c.count).unwrap_or(0);

    // page of docs
    let mut resp = get_db()
        .query(
            "SELECT * FROM users WHERE status >= 1 ORDER BY created_at DESC LIMIT $ps START $skip",
        )
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;

    let mut items = Vec::with_capacity(docs.len());
    for d in docs {
        let keywords = topic_db::get_keywords_by_user_id(&d.id)
            .await
            .unwrap_or_else(|e| {
                leptos::logging::error!("get_keywords_by_user_id: {e}");
                vec![]
            });
        let topics = topic_db::get_topics_by_user_id(&d.id)
            .await
            .unwrap_or_else(|e| {
                leptos::logging::error!("get_topics_by_user_id: {e}");
                vec![]
            });
        items.push(UserSummary {
            id: rid_str(&d.id),
            username: d.username,
            created_at: common::ymd8(&d.created_at),
            updated_at: common::ymd8(&d.updated_at),
            status: d.status,
            keywords,
            topics,
        });
    }

    Ok(UsersResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

/// Look up a single user by username, including keywords, topics, and
/// rendered introduction HTML.
pub async fn get_user_by_username(username: &str) -> Result<Option<User>, String> {
    let mut resp = get_db()
        .query("SELECT * FROM users WHERE username = $username LIMIT 1")
        .bind(("username", username.to_owned()))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;

    let Some(d) = docs.into_iter().next() else {
        return Ok(None);
    };

    let keywords = topic_db::get_keywords_by_user_id(&d.id)
        .await
        .unwrap_or_else(|e| {
            leptos::logging::error!("get_keywords_by_user_id: {e}");
            vec![]
        });
    let topics = topic_db::get_topics_by_user_id(&d.id)
        .await
        .unwrap_or_else(|e| {
            leptos::logging::error!("get_topics_by_user_id: {e}");
            vec![]
        });

    Ok(Some(User {
        id: rid_str(&d.id),
        username: d.username,
        email: d.email,
        introduction_html: render_md(&d.introduction),
        introduction: d.introduction,
        created_at: common::ymd8(&d.created_at),
        updated_at: common::ymd8(&d.updated_at),
        status: d.status,
        keywords,
        topics,
    }))
}

/// Raw user document lookup by id (accepts full RecordId).
pub async fn get_user_doc_by_id(rid: &RecordId) -> Result<Option<UserDoc>, String> {
    get_db().select(rid).await.map_err(|e| e.to_string())
}

/// Authenticate by email or username + password.
/// Returns `AuthUser` on success or a typed error string.
pub async fn sign_in(signature: &str, password: &str) -> Result<AuthUser, String> {
    let mut resp = if signature.contains('@') {
        get_db()
            .query("SELECT * FROM users WHERE email = $sig LIMIT 1")
            .bind(("sig", signature.to_owned()))
            .await
    } else {
        get_db()
            .query("SELECT * FROM users WHERE username = $sig LIMIT 1")
            .bind(("sig", signature.to_owned()))
            .await
    }
    .map_err(|e| e.to_string())?;

    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;

    let user = docs
        .into_iter()
        .next()
        .ok_or_else(|| "sign_in_incorrect".to_string())?;

    match user.status {
        1..=10 => {}
        0 => {
            let uid = rid_str(&user.id);
            return Err(format!("sign_in_not_activation:{}", record_key(&uid)));
        }
        -1 => return Err("sign_in_banned".to_string()),
        _ => return Err("sign_in_security_problem".to_string()),
    }

    if !auth_mod::verify_credential(&user.username, password, &user.cred) {
        return Err("sign_in_incorrect".to_string());
    }

    let token = auth_mod::encode_jwt(&user.email, &user.username)?;
    Ok(AuthUser {
        username: user.username,
        token,
    })
}

/// Register a new user (status=0).  Returns `(user_id, username)`.
pub async fn register_user(data: RegisterData) -> Result<(String, String), String> {
    let username = data.username.trim().to_lowercase();
    let email = data.email.trim().to_lowercase();

    // uniqueness check
    let mut resp = get_db()
        .query("SELECT count() FROM users WHERE username = $username OR email = $email GROUP ALL")
        .bind(("username", username.clone()))
        .bind(("email", email.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = resp.take(0).map_err(|e| e.to_string())?;
    if counts.first().map(|c| c.count).unwrap_or(0) > 0 {
        return Err("register_exist".to_string());
    }

    let cred = auth_mod::hash_credential(&username, &data.password);
    // NOTE: kept `.query()` over `db.create().content()` because
    // `time::now()` is a SurrealQL function — not expressible as a Rust literal.
    let mut resp = get_db()
        .query(
            "CREATE users CONTENT { \
                username: $username, \
                email: $email, \
                cred: $cred, \
                introduction: $introduction, \
                created_at: time::now(), \
                updated_at: time::now(), \
                status: 0 \
            }",
        )
        .bind(("username", username.clone()))
        .bind(("email", email.clone()))
        .bind(("cred", cred))
        .bind(("introduction", data.introduction.trim().to_owned()))
        .await
        .map_err(|e| e.to_string())?;
    let created: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;
    let user_doc = created.into_iter().next().ok_or("failed to create user")?;
    let uid_str = rid_str(&user_doc.id);

    // optional topics
    if !data.topics.trim().is_empty() {
        let tids = topic_db::create_topics_from_names(&data.topics).await?;
        topic_db::link_topics_to_user(&uid_str, tids).await?;
    }

    Ok((uid_str, username))
}

/// Set status 0 → 1.  Returns the user's username if activation happened.
pub async fn activate_user(rid: &RecordId) -> Result<Option<String>, String> {
    let doc: Option<UserDoc> = get_db().select(rid).await.map_err(|e| e.to_string())?;

    let Some(u) = doc else {
        return Ok(None);
    };

    if u.status == 0 {
        get_db()
            .query("UPDATE $rid SET status = 1, updated_at = time::now()")
            .bind(("rid", rid.clone()))
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(Some(u.username))
}

/// Convenience lookup returning `(email, username)`.
pub async fn get_user_email_username(rid: &RecordId) -> Result<Option<(String, String)>, String> {
    Ok(get_user_doc_by_id(rid)
        .await?
        .map(|u| (u.email, u.username)))
}
