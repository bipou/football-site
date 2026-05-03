use serde::Serialize;
use bson::{doc, oid::ObjectId, DateTime as BsonDt};
use chrono::{DateTime, FixedOffset, Utc};
use mongodb::Collection;
use serde::Deserialize;

use crate::models::{AuthUser, PageInfo, User, UserSummary, UsersResult};
use crate::server::{auth as auth_mod, db::get_db, topic_db};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserDoc {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    pub cred: String,
    pub nickname: String,
    #[serde(default)] pub phone_number: String,
    #[serde(default)] pub phone_public: bool,
    #[serde(default)] pub im_account: String,
    #[serde(default)] pub im_public: bool,
    #[serde(default)] pub website: String,
    #[serde(default)] pub introduction: String,
    pub created_at: BsonDt,
    pub updated_at: BsonDt,
    pub status: i8,
}

fn fmt8(bdt: BsonDt) -> String {
    let dt: DateTime<Utc> = bdt.into();
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%Y-%m-%d %H:%M:%S%:z").to_string()
}

fn render_md(md: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    let mut out = String::new();
    html::push_html(&mut out, Parser::new_ext(md, opts));
    out
}

fn page_size() -> i64 {
    std::env::var("PAGE_SIZE").ok().and_then(|s| s.parse().ok()).unwrap_or(12)
}

fn make_page_info(from: i64, ps: i64, total: u64) -> PageInfo {
    let tp = ((total as f64 / ps as f64).ceil() as u32).max(1);
    PageInfo { current_page: from as u32, total_pages: tp, total_count: total, first_cursor: String::new(), last_cursor: String::new(), has_previous: from > 1, has_next: (from as u32) < tp }
}

pub async fn get_users(from: i64) -> Result<UsersResult, String> {
    let ps   = page_size();
    let coll: Collection<UserDoc> = get_db().collection("users");
    let filt = doc! { "status": { "$gte": 1i32 } };
    let total = coll.count_documents(filt.clone()).await.map_err(|e| e.to_string())?;
    let skip  = ((from - 1) * ps).max(0) as u64;
    let mut cur = coll.find(filt).sort(doc! { "created_at": -1 }).skip(skip).limit(ps).await.map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        let d = cur.deserialize_current().map_err(|e| e.to_string())?;
        let uid = d.id.to_hex();
        let keywords = topic_db::get_keywords_by_user_id(&uid).await.unwrap_or_default();
        let topics   = topic_db::get_topics_by_user_id(&uid).await.unwrap_or_default();
        items.push(UserSummary { id: uid, username: d.username, nickname: d.nickname, created_at: fmt8(d.created_at), status: d.status, keywords, topics });
    }
    Ok(UsersResult { page_info: make_page_info(from, ps, total), items })
}

pub async fn get_user_by_username(username: &str) -> Result<Option<User>, String> {
    let coll: Collection<UserDoc> = get_db().collection("users");
    let Some(d) = coll.find_one(doc! { "username": username }).await.map_err(|e| e.to_string())? else { return Ok(None) };
    let uid = d.id.to_hex();
    let keywords = topic_db::get_keywords_by_user_id(&uid).await.unwrap_or_default();
    let topics   = topic_db::get_topics_by_user_id(&uid).await.unwrap_or_default();
    Ok(Some(User {
        id: uid, username: d.username, email: d.email, nickname: d.nickname,
        phone_number: d.phone_number, phone_public: d.phone_public,
        im_account: d.im_account, im_public: d.im_public, website: d.website,
        introduction_html: render_md(&d.introduction),
        introduction: d.introduction,
        created_at: fmt8(d.created_at), updated_at: fmt8(d.updated_at),
        status: d.status, keywords, topics,
    }))
}

pub async fn get_user_doc_by_id(id: &str) -> Result<Option<UserDoc>, String> {
    let oid  = ObjectId::parse_str(id).map_err(|e| e.to_string())?;
    let coll: Collection<UserDoc> = get_db().collection("users");
    coll.find_one(doc! { "_id": oid }).await.map_err(|e| e.to_string())
}

pub async fn sign_in(signature: &str, password: &str) -> Result<AuthUser, String> {
    let coll: Collection<UserDoc> = get_db().collection("users");
    let doc = if signature.contains('@') {
        coll.find_one(doc! { "email": signature }).await.map_err(|e| e.to_string())?
    } else {
        coll.find_one(doc! { "username": signature }).await.map_err(|e| e.to_string())?
    };

    let user = doc.ok_or_else(|| "sign_in_incorrect".to_string())?;

    match user.status {
        1..=10 => {}
        0  => return Err(format!("sign_in_not_activation:{}", user.id.to_hex())),
        -1 => return Err("sign_in_banned".to_string()),
        _  => return Err("sign_in_security_problem".to_string()),
    }

    if !auth_mod::verify_credential(&user.username, password, &user.cred) {
        return Err("sign_in_incorrect".to_string());
    }

    let token = auth_mod::encode_jwt(&user.email, &user.username)?;
    Ok(AuthUser { username: user.username, token })
}

pub struct RegisterData {
    pub username: String, pub email: String, pub password: String,
    pub nickname: String, pub phone_number: String, pub phone_public: bool,
    pub im_account: String, pub im_public: bool,
    pub website: String, pub introduction: String, pub topics: String,
}

pub async fn register_user(data: RegisterData) -> Result<(String, String, String), String> {
    let db   = get_db();
    let coll: Collection<bson::Document> = db.collection("users");
    let username = data.username.trim().to_lowercase();
    let email    = data.email.trim().to_lowercase();

    if coll.find_one(doc! { "$or": [{ "username": &username }, { "email": &email }] }).await.map_err(|e| e.to_string())?.is_some() {
        return Err("register_exist".to_string());
    }

    let cred = auth_mod::hash_credential(&username, &data.password);
    let now  = BsonDt::now();
    let nickname = data.nickname.trim().to_string();
    let res  = coll.insert_one(doc! {
        "username": &username, "email": &email, "cred": cred,
        "nickname": &nickname, "phone_number": data.phone_number.trim(),
        "phone_public": data.phone_public, "im_account": data.im_account.trim(),
        "im_public": data.im_public, "website": data.website.trim(),
        "introduction": data.introduction.trim(),
        "created_at": now, "updated_at": now, "status": 0i32,
    }).await.map_err(|e| e.to_string())?;

    let uid_str = res.inserted_id.as_object_id().ok_or("no inserted id")?.to_hex();
    let uid_oid = ObjectId::parse_str(&uid_str).unwrap();

    if !data.topics.trim().is_empty() {
        let tids = topic_db::create_topics_from_names(&data.topics).await?;
        topic_db::link_topics_to_user(uid_oid, tids).await?;
    }
    // Return (user_id, nickname, username) for the email send
    Ok((uid_str, nickname, username))
}

pub async fn activate_user(user_id: &str) -> Result<Option<String>, String> {
    let oid  = ObjectId::parse_str(user_id).map_err(|e| e.to_string())?;
    let coll: Collection<UserDoc> = get_db().collection("users");
    let Some(u) = coll.find_one(doc! { "_id": oid }).await.map_err(|e| e.to_string())? else { return Ok(None) };
    if u.status == 0 {
        coll.update_one(doc! { "_id": oid }, doc! { "$set": { "status": 1i32, "updated_at": BsonDt::now() } }).await.map_err(|e| e.to_string())?;
    }
    Ok(Some(u.nickname))
}

pub async fn get_user_email_nickname(user_id: &str) -> Result<Option<(String, String, String)>, String> {
    Ok(get_user_doc_by_id(user_id).await?.map(|u| (u.email, u.nickname, u.username)))
}
