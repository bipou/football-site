use crate::utils::common;
use serde::Deserialize;
use surrealdb::types::{Datetime, RecordId, SurrealValue};

use crate::models::{AuthUser, User, UserSummary, UsersResult};
use crate::server::{auth as auth_mod, db::get_db, topic_db};
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
    nickname: String,
    #[serde(default)]
    phone_number: String,
    #[serde(default)]
    phone_public: bool,
    #[serde(default)]
    im_account: String,
    #[serde(default)]
    im_public: bool,
    #[serde(default)]
    website: String,
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
    pub nickname: String,
    pub phone_number: String,
    pub phone_public: bool,
    pub im_account: String,
    pub im_public: bool,
    pub website: String,
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
    let sql = format!(
        "SELECT * FROM users WHERE status >= 1 ORDER BY created_at DESC LIMIT {ps} START {skip}"
    );
    let mut resp = get_db().query(&sql).await.map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;

    let mut items = Vec::with_capacity(docs.len());
    for d in docs {
        let uid = common::id_only(&d.id);
        let keywords = topic_db::get_keywords_by_user_id(&uid)
            .await
            .unwrap_or_default();
        let topics = topic_db::get_topics_by_user_id(&uid)
            .await
            .unwrap_or_default();
        items.push(UserSummary {
            id: uid,
            username: d.username,
            created_at: common::ymdhmsz8(&d.created_at),
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

    let uid = common::id_only(&d.id);
    let keywords = topic_db::get_keywords_by_user_id(&uid)
        .await
        .unwrap_or_default();
    let topics = topic_db::get_topics_by_user_id(&uid)
        .await
        .unwrap_or_default();

    Ok(Some(User {
        id: uid,
        username: d.username,
        email: d.email,
        introduction_html: render_md(&d.introduction),
        introduction: d.introduction,
        created_at: common::ymdhmsz8(&d.created_at),
        updated_at: common::ymdhmsz8(&d.updated_at),
        status: d.status,
        keywords,
        topics,
    }))
}

/// Raw user document lookup by id (accepts plain id or `users:xxx`).
pub async fn get_user_doc_by_id(id: &str) -> Result<Option<UserDoc>, String> {
    let rid = common::record_id("users", id);
    let sql = format!("SELECT * FROM {rid}");
    let mut resp = get_db().query(&sql).await.map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().next())
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
            return Err(format!(
                "sign_in_not_activation:{}",
                common::id_only(&user.id)
            ));
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

/// Register a new user (status=0).  Returns `(user_id, nickname, username)`.
pub async fn register_user(data: RegisterData) -> Result<(String, String, String), String> {
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
    let nickname = data.nickname.trim().to_string();

    let mut resp = get_db()
        .query(
            "CREATE users CONTENT { \
                username: $username, \
                email: $email, \
                cred: $cred, \
                nickname: $nickname, \
                phone_number: $phone_number, \
                phone_public: $phone_public, \
                im_account: $im_account, \
                im_public: $im_public, \
                website: $website, \
                introduction: $introduction, \
                created_at: time::now(), \
                updated_at: time::now(), \
                status: 0 \
            }",
        )
        .bind(("username", username.clone()))
        .bind(("email", email.clone()))
        .bind(("cred", cred))
        .bind(("nickname", nickname.clone()))
        .bind(("phone_number", data.phone_number.trim().to_owned()))
        .bind(("phone_public", data.phone_public))
        .bind(("im_account", data.im_account.trim().to_owned()))
        .bind(("im_public", data.im_public))
        .bind(("website", data.website.trim().to_owned()))
        .bind(("introduction", data.introduction.trim().to_owned()))
        .await
        .map_err(|e| e.to_string())?;
    let created: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;
    let user_doc = created.into_iter().next().ok_or("failed to create user")?;
    let uid_str = common::id_only(&user_doc.id);

    // optional topics
    if !data.topics.trim().is_empty() {
        let tids = topic_db::create_topics_from_names(&data.topics).await?;
        topic_db::link_topics_to_user(&uid_str, tids).await?;
    }

    Ok((uid_str, nickname, username))
}

/// Set status 0 → 1.  Returns the user's nickname if activation happened.
pub async fn activate_user(user_id: &str) -> Result<Option<String>, String> {
    let rid = common::record_id("users", user_id);

    let sql = format!("SELECT * FROM {rid}");
    let mut resp = get_db().query(&sql).await.map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;

    let Some(u) = docs.into_iter().next() else {
        return Ok(None);
    };

    if u.status == 0 {
        get_db()
            .query("UPDATE $rid SET status = 1, updated_at = time::now()")
            .bind(("rid", rid))
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(Some(u.nickname))
}

/// Convenience lookup returning `(email, nickname, username)`.
pub async fn get_user_email_username(
    user_id: &str,
) -> Result<Option<(String, String)>, String> {
    Ok(get_user_doc_by_id(user_id)
        .await?
        .map(|u| (u.email, u.username)))
}
