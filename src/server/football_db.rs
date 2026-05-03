use bson::{doc, oid::ObjectId, DateTime as BsonDt};
use chrono::{DateTime, Duration, FixedOffset, TimeZone, Timelike, Utc};
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::models::{Football, FootballLine, FootballOver, FootballsResult, PageInfo};
use crate::server::{category_db, db::get_db, topic_db};

// ── BSON document types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[derive(Serialize)]
struct FootballDoc {
    #[serde(rename = "_id")]
    id: ObjectId,
    category_id: ObjectId,
    season: String,
    home_team: String,
    away_team: String,
    kick_off_at: BsonDt,
    created_at: BsonDt,
    updated_at: BsonDt,
    #[serde(default)]
    hits: i64,
    #[serde(default)]
    stars: i64,
    status: i8,
}

#[derive(Debug, Deserialize)]
struct FootballLineDoc {
    #[serde(rename = "_id")]
    id: ObjectId,
    win: String,
    draw: String,
    loss: String,
    kind: u8,
    created_at: BsonDt,
}

#[derive(Debug, Deserialize)]
struct FootballOverDoc {
    #[serde(rename = "_id")]
    id: ObjectId,
    s: String,
    wdl: String,
    tg: String,
    gd: String,
    kind: u8,
    created_at: BsonDt,
}

// ── Datetime helpers ───────────────────────────────────────────────────────────

fn mdhm(bdt: BsonDt) -> String {
    let dt: DateTime<Utc> = bdt.into();
    dt.format("%m-%d %H:%M").to_string()
}

fn mdhm8(bdt: BsonDt) -> String {
    let dt: DateTime<Utc> = bdt.into();
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%m-%d %H:%M").to_string()
}

fn ymdhmsz8(bdt: BsonDt) -> String {
    let dt: DateTime<Utc> = bdt.into();
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%Y-%m-%d %H:%M:%S%:z").to_string()
}

// ── Conversions ────────────────────────────────────────────────────────────────

fn line_into(d: FootballLineDoc) -> FootballLine {
    FootballLine { id: d.id.to_hex(), win: d.win, draw: d.draw, loss: d.loss, kind: d.kind, created_at: ymdhmsz8(d.created_at) }
}

fn over_into(d: FootballOverDoc) -> FootballOver {
    FootballOver { id: d.id.to_hex(), s: d.s, wdl: d.wdl, tg: d.tg, gd: d.gd, kind: d.kind, created_at: ymdhmsz8(d.created_at) }
}

/// Return [first, last] pair — or fewer items if < 2 available.
fn il_pair<T: Clone>(v: Vec<T>) -> Vec<T> {
    match v.len() {
        0 => vec![],
        1 => v,
        n => vec![v[0].clone(), v[n - 1].clone()],
    }
}

// ── Internal fetch helpers ─────────────────────────────────────────────────────

async fn fetch_lines(fid: ObjectId, kind: u8) -> Result<Vec<FootballLine>, String> {
    let coll: Collection<FootballLineDoc> = get_db().collection("footballs_lines");
    let mut cur = coll.find(doc! { "football_id": fid, "kind": kind as i32 }).sort(doc! { "created_at": 1 }).await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        out.push(line_into(cur.deserialize_current().map_err(|e| e.to_string())?));
    }
    Ok(out)
}

async fn fetch_overs(fid: ObjectId, kind: u8) -> Result<Vec<FootballOver>, String> {
    let coll: Collection<FootballOverDoc> = get_db().collection("footballs_overs");
    let mut cur = coll.find(doc! { "football_id": fid, "kind": kind as i32 }).sort(doc! { "created_at": 1 }).await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        out.push(over_into(cur.deserialize_current().map_err(|e| e.to_string())?));
    }
    Ok(out)
}

async fn enrich(doc: FootballDoc) -> Result<Football, String> {
    let fid = doc.id;
    let lines    = fetch_lines(fid, 0).await?;
    let calcs    = fetch_overs(fid, 0).await?;
    let officials = fetch_overs(fid, 1).await?;
    let topics   = topic_db::get_topics_by_football_id(&fid.to_hex()).await?;
    let category = category_db::get_category_by_id(&doc.category_id.to_hex()).await?;

    Ok(Football {
        id: fid.to_hex(),
        category_id: doc.category_id.to_hex(),
        season: doc.season,
        home_team: doc.home_team,
        away_team: doc.away_team,
        kick_off_at_mdhm:  mdhm(doc.kick_off_at),
        kick_off_at_mdhm8: mdhm8(doc.kick_off_at),
        created_at: ymdhmsz8(doc.created_at),
        updated_at: ymdhmsz8(doc.updated_at),
        hits:  doc.hits.max(0) as u64,
        stars: doc.stars.max(0) as u64,
        status: doc.status,
        il_odds:      il_pair(lines),
        il_calc_over: il_pair(calcs),
        football_over: officials.into_iter().last(),
        category,
        topics,
    })
}

// ── Pagination helper ──────────────────────────────────────────────────────────

fn page_size() -> i64 {
    std::env::var("PAGE_SIZE").ok().and_then(|s| s.parse().ok()).unwrap_or(12)
}

fn make_page_info(from: i64, ps: i64, total: u64) -> PageInfo {
    let tp = ((total as f64 / ps as f64).ceil() as u32).max(1);
    PageInfo {
        current_page: from as u32,
        total_pages:  tp,
        total_count:  total,
        first_cursor: String::new(),
        last_cursor:  String::new(),
        has_previous: from > 1,
        has_next:     (from as u32) < tp,
    }
}

// ── Public API ─────────────────────────────────────────────────────────────────

pub async fn get_footballs_in_position(position: &str, limit: i64) -> Result<Vec<Football>, String> {
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let now_utc = Utc::now();
    let now8 = now_utc.with_timezone(&tz8);
    let cutoff: i64 = if now8.hour() >= 11 { 0 } else { 1 };
    let day_off = match position {
        "jt" => -cutoff,
        "zt" => -(cutoff + 1),
        _    => return Err("position must be 'jt' or 'zt'".into()),
    };

    let target  = now8.date_naive() + Duration::days(day_off);
    let start_l = tz8.from_local_datetime(&target.and_hms_opt(11, 0, 0).unwrap()).single().ok_or("ambiguous datetime")?;
    let end_l   = start_l + Duration::days(1);

    let start_bdt = BsonDt::from_chrono(start_l.with_timezone(&Utc));
    let end_bdt   = BsonDt::from_chrono(end_l.with_timezone(&Utc));

    let coll: Collection<FootballDoc> = get_db().collection("footballs");
    let mut cur = coll
        .find(doc! { "kick_off_at": { "$gte": start_bdt, "$lt": end_bdt }, "status": { "$gte": 0i32 } })
        .sort(doc! { "kick_off_at": -1, "updated_at": -1 })
        .limit(limit)
        .await
        .map_err(|e| e.to_string())?;

    let mut docs = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        docs.push(cur.deserialize_current().map_err(|e| e.to_string())?);
    }
    let mut out = Vec::new();
    for d in docs { out.push(enrich(d).await?); }
    Ok(out)
}

pub async fn get_football_by_id(id: &str) -> Result<Option<Football>, String> {
    let oid  = ObjectId::parse_str(id).map_err(|e| e.to_string())?;
    let coll: Collection<FootballDoc> = get_db().collection("footballs");
    match coll.find_one(doc! { "_id": oid }).await.map_err(|e| e.to_string())? {
        Some(d) => Ok(Some(enrich(d).await?)),
        None    => Ok(None),
    }
}

pub async fn get_random_football_id() -> Result<Option<String>, String> {
    let coll = get_db().collection::<bson::Document>("footballs");
    let pipeline = vec![
        doc! { "$match": { "status": { "$gte": 1i32 } } },
        doc! { "$sample": { "size": 1i32 } },
        doc! { "$project": { "_id": 1i32 } },
    ];
    let mut cur = coll.aggregate(pipeline).await.map_err(|e| e.to_string())?;
    if cur.advance().await.map_err(|e| e.to_string())? {
        if let Ok(id) = cur.current().get_object_id("_id") {
            return Ok(Some(id.to_hex()));
        }
    }
    Ok(None)
}

pub async fn get_footballs(from: i64, status_min: i8, status_max: i8) -> Result<FootballsResult, String> {
    let ps   = page_size();
    let coll: Collection<FootballDoc> = get_db().collection("footballs");
    let filt = doc! { "status": { "$gte": status_min as i32, "$lte": status_max as i32 } };
    let total = coll.count_documents(filt.clone()).await.map_err(|e| e.to_string())?;
    let skip  = ((from - 1) * ps).max(0) as u64;
    let mut cur = coll.find(filt).sort(doc! { "kick_off_at": -1, "updated_at": -1 }).skip(skip).limit(ps).await.map_err(|e| e.to_string())?;
    let mut docs = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? { docs.push(cur.deserialize_current().map_err(|e| e.to_string())?); }
    let mut items = Vec::new();
    for d in docs { items.push(enrich(d).await?); }
    Ok(FootballsResult { page_info: make_page_info(from, ps, total), items })
}

pub async fn get_footballs_by_category(category_id: &str, from: i64) -> Result<FootballsResult, String> {
    let cid  = ObjectId::parse_str(category_id).map_err(|e| e.to_string())?;
    let ps   = page_size();
    let coll: Collection<FootballDoc> = get_db().collection("footballs");
    let filt = doc! { "category_id": cid, "status": { "$gte": 1i32 } };
    let total = coll.count_documents(filt.clone()).await.map_err(|e| e.to_string())?;
    let skip  = ((from - 1) * ps).max(0) as u64;
    let mut cur = coll.find(filt).sort(doc! { "kick_off_at": -1, "updated_at": -1 }).skip(skip).limit(ps).await.map_err(|e| e.to_string())?;
    let mut docs = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? { docs.push(cur.deserialize_current().map_err(|e| e.to_string())?); }
    let mut items = Vec::new();
    for d in docs { items.push(enrich(d).await?); }
    Ok(FootballsResult { page_info: make_page_info(from, ps, total), items })
}

pub async fn get_footballs_by_topic(topic_id: &str, from: i64) -> Result<FootballsResult, String> {
    let tid = ObjectId::parse_str(topic_id).map_err(|e| e.to_string())?;
    let db  = get_db();

    #[derive(Deserialize)]
    struct Rel { football_id: Option<ObjectId> }
    let rel: Collection<Rel> = db.collection("topics_relevant");
    let mut cur = rel.find(doc! { "topic_id": tid, "football_id": { "$exists": true } }).await.map_err(|e| e.to_string())?;
    let mut fids: Vec<ObjectId> = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        let r = cur.deserialize_current().map_err(|e| e.to_string())?;
        if let Some(fid) = r.football_id { if !fids.contains(&fid) { fids.push(fid); } }
    }

    let ps    = page_size();
    let total = fids.len() as u64;
    let skip  = ((from - 1) * ps).max(0) as usize;
    let page_fids: Vec<ObjectId> = fids.into_iter().skip(skip).take(ps as usize).collect();

    let coll: Collection<FootballDoc> = db.collection("footballs");
    let mut cur = coll.find(doc! { "_id": { "$in": page_fids }, "status": { "$gte": 1i32 } }).sort(doc! { "kick_off_at": -1 }).await.map_err(|e| e.to_string())?;
    let mut docs = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? { docs.push(cur.deserialize_current().map_err(|e| e.to_string())?); }
    let mut items = Vec::new();
    for d in docs { items.push(enrich(d).await?); }
    Ok(FootballsResult { page_info: make_page_info(from, ps, total), items })
}

pub async fn get_footballs_admin(from: i64) -> Result<FootballsResult, String> {
    let ps   = page_size();
    let coll: Collection<FootballDoc> = get_db().collection("footballs");
    let total = coll.count_documents(doc! {}).await.map_err(|e| e.to_string())?;
    let skip  = ((from - 1) * ps).max(0) as u64;
    let mut cur = coll.find(doc! {}).sort(doc! { "updated_at": -1 }).skip(skip).limit(ps).await.map_err(|e| e.to_string())?;
    let mut docs = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? { docs.push(cur.deserialize_current().map_err(|e| e.to_string())?); }
    let mut items = Vec::new();
    for d in docs { items.push(enrich(d).await?); }
    Ok(FootballsResult { page_info: make_page_info(from, ps, total), items })
}

pub async fn update_football_status(id: &str, status: i8) -> Result<(), String> {
    let oid = ObjectId::parse_str(id).map_err(|e| e.to_string())?;
    get_db().collection::<bson::Document>("footballs")
        .update_one(doc! { "_id": oid }, doc! { "$set": { "status": status as i32, "updated_at": BsonDt::now() } })
        .await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn increment_hits(id: &str) -> Result<(), String> {
    let oid = ObjectId::parse_str(id).map_err(|e| e.to_string())?;
    get_db().collection::<bson::Document>("footballs")
        .update_one(doc! { "_id": oid }, doc! { "$inc": { "hits": 1i64 } })
        .await.map_err(|e| e.to_string())?;
    Ok(())
}
