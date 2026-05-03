use chrono::{FixedOffset, Utc};
use serde::Deserialize;
use surrealdb::sql::{Datetime, Thing};

use crate::models::{AuthUser, PageInfo, User, UserSummary, UsersResult};
use crate::server::{auth as auth_mod, db::get_db, topic_db};

// ── helpers ──────────────────────────────────────────────────────────────

fn id_only(t: &Thing) -> String {
    t.id.to_string()
}

fn record_id(table: &str, id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("{}:{}", table, id)
    }
}

fn fmt8(dt: &Datetime) -> String {
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.0.with_timezone(&tz8)
        .format("%Y-%m-%d %H:%M:%S%:z")
        .to_string()
}

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

fn page_size() -> i64 {
    std::env::var("PAGE_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12)
}

fn make_page_info(from: i64, ps: i64, total: u64) -> PageInfo {
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

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn now_utc() -> chrono::DateTime<Utc> {
    Utc::now()
}

fn fmt_dt(dt: &chrono::DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// ── document structs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UserDoc {
    id: Thing,
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

#[derive(Debug, Deserialize)]
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
    let ps = page_size();
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
        let uid = id_only(&d.id);
        let keywords = topic_db::get_keywords_by_user_id(&uid)
            .await
            .unwrap_or_default();
        let topics = topic_db::get_topics_by_user_id(&uid)
            .await
            .unwrap_or_default();
        items.push(UserSummary {
            id: uid,
            username: d.username,
            nickname: d.nickname,
            created_at: fmt8(&d.created_at),
            status: d.status,
            keywords,
            topics,
        });
    }

    Ok(UsersResult {
        page_info: make_page_info(from, ps, total),
        items,
    })
}

/// Look up a single user by username, including keywords, topics, and
/// rendered introduction HTML.
pub async fn get_user_by_username(username: &str) -> Result<Option<User>, String> {
    let sql = format!(
        "SELECT * FROM users WHERE username = '{}' LIMIT 1",
        esc(username)
    );
    let mut resp = get_db().query(&sql).await.map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;

    let Some(d) = docs.into_iter().next() else {
        return Ok(None);
    };

    let uid = id_only(&d.id);
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
        nickname: d.nickname,
        phone_number: d.phone_number,
        phone_public: d.phone_public,
        im_account: d.im_account,
        im_public: d.im_public,
        website: d.website,
        introduction_html: render_md(&d.introduction),
        introduction: d.introduction,
        created_at: fmt8(&d.created_at),
        updated_at: fmt8(&d.updated_at),
        status: d.status,
        keywords,
        topics,
    }))
}

/// Raw user document lookup by id (accepts plain id or `users:xxx`).
pub async fn get_user_doc_by_id(id: &str) -> Result<Option<UserDoc>, String> {
    let rid = record_id("users", id);
    let sql = format!("SELECT * FROM {rid}");
    let mut resp = get_db().query(&sql).await.map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().next())
}

/// Authenticate by email or username + password.
/// Returns `AuthUser` on success or a typed error string.
pub async fn sign_in(signature: &str, password: &str) -> Result<AuthUser, String> {
    let sql = if signature.contains('@') {
        format!(
            "SELECT * FROM users WHERE email = '{}' LIMIT 1",
            esc(signature)
        )
    } else {
        format!(
            "SELECT * FROM users WHERE username = '{}' LIMIT 1",
            esc(signature)
        )
    };

    let mut resp = get_db().query(&sql).await.map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;

    let user = docs
        .into_iter()
        .next()
        .ok_or_else(|| "sign_in_incorrect".to_string())?;

    match user.status {
        1..=10 => {}
        0 => return Err(format!("sign_in_not_activation:{}", id_only(&user.id))),
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
    let check_sql = format!(
        "SELECT count() FROM users WHERE username = '{}' OR email = '{}' GROUP ALL",
        esc(&username),
        esc(&email)
    );
    let mut resp = get_db()
        .query(&check_sql)
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = resp.take(0).map_err(|e| e.to_string())?;
    if counts.first().map(|c| c.count).unwrap_or(0) > 0 {
        return Err("register_exist".to_string());
    }

    let cred = auth_mod::hash_credential(&username, &data.password);
    let now = now_utc();
    let now_str = fmt_dt(&now);
    let nickname = data.nickname.trim().to_string();

    let insert_sql = format!(
        "CREATE users CONTENT {{ \
            username: '{}', \
            email: '{}', \
            cred: '{}', \
            nickname: '{}', \
            phone_number: '{}', \
            phone_public: {}, \
            im_account: '{}', \
            im_public: {}, \
            website: '{}', \
            introduction: '{}', \
            created_at: <datetime>'{}', \
            updated_at: <datetime>'{}', \
            status: 0 \
        }}",
        esc(&username),
        esc(&email),
        esc(&cred),
        esc(&nickname),
        esc(data.phone_number.trim()),
        data.phone_public,
        esc(data.im_account.trim()),
        data.im_public,
        esc(data.website.trim()),
        esc(data.introduction.trim()),
        now_str,
        now_str,
    );

    let mut resp = get_db()
        .query(&insert_sql)
        .await
        .map_err(|e| e.to_string())?;
    let created: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;
    let user_doc = created.into_iter().next().ok_or("failed to create user")?;
    let uid_str = id_only(&user_doc.id);

    // optional topics
    if !data.topics.trim().is_empty() {
        let tids = topic_db::create_topics_from_names(&data.topics).await?;
        topic_db::link_topics_to_user(&uid_str, tids).await?;
    }

    Ok((uid_str, nickname, username))
}

/// Set status 0 → 1.  Returns the user's nickname if activation happened.
pub async fn activate_user(user_id: &str) -> Result<Option<String>, String> {
    let rid = record_id("users", user_id);

    let sql = format!("SELECT * FROM {rid}");
    let mut resp = get_db().query(&sql).await.map_err(|e| e.to_string())?;
    let docs: Vec<UserDoc> = resp.take(0).map_err(|e| e.to_string())?;

    let Some(u) = docs.into_iter().next() else {
        return Ok(None);
    };

    if u.status == 0 {
        let now_str = fmt_dt(&Utc::now());
        let update_sql = format!(
            "UPDATE {rid} SET status = 1, updated_at = <datetime>'{}'",
            now_str
        );
        get_db()
            .query(&update_sql)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(Some(u.nickname))
}

/// Convenience lookup returning `(email, nickname, username)`.
pub async fn get_user_email_nickname(
    user_id: &str,
) -> Result<Option<(String, String, String)>, String> {
    Ok(get_user_doc_by_id(user_id)
        .await?
        .map(|u| (u.email, u.nickname, u.username)))
}
