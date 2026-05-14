use crate::utils::common;
use crate::utils::constant;
use chrono::{Duration, FixedOffset, TimeZone, Timelike, Utc};
use serde::Deserialize;
use surrealdb::types::{Datetime as Sdt, RecordId, SurrealValue};

use crate::models::{Football, FootballLine, FootballOver, FootballsResult};
use crate::server::{category_db, db::get_db, topic_db};

// ── Helpers ────────────────────────────────────────────────────────────────────

// ── Datetime formatters ────────────────────────────────────────────────────────

fn mdhm(dt: &Sdt) -> String {
    dt.format("%m-%d %H:%M").to_string()
}

fn mdhm8(dt: &Sdt) -> String {
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&tz8).format("%m-%d %H:%M").to_string()
}

// ── Doc structs ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, SurrealValue)]
struct FootballDoc {
    id: RecordId,
    category_id: String,
    season: String,
    home_team: String,
    away_team: String,
    kick_off_at: Sdt,
    created_at: Sdt,
    updated_at: Sdt,
    #[serde(default)]
    hits: i64,
    #[serde(default)]
    stars: i64,
    status: i8,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct FootballLineDoc {
    id: RecordId,
    win: String,
    draw: String,
    loss: String,
    kind: u8,
    created_at: Sdt,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct FootballOverDoc {
    id: RecordId,
    s: String,
    wdl: String,
    tg: String,
    gd: String,
    kind: u8,
    created_at: Sdt,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct CountResult {
    count: u64,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct IdOnly {
    id: RecordId,
}

// ── Conversions ────────────────────────────────────────────────────────────────

fn line_into(d: FootballLineDoc) -> FootballLine {
    FootballLine {
        id: common::id_only(&d.id),
        win: d.win,
        draw: d.draw,
        loss: d.loss,
        kind: d.kind,
        created_at: common::ymdhmsz8(&d.created_at),
    }
}

fn over_into(d: FootballOverDoc) -> FootballOver {
    FootballOver {
        id: common::id_only(&d.id),
        s: d.s,
        wdl: d.wdl,
        tg: d.tg,
        gd: d.gd,
        kind: d.kind,
        created_at: common::ymdhmsz8(&d.created_at),
    }
}

fn il_pair<T: Clone>(v: Vec<T>) -> Vec<T> {
    match v.len() {
        0 => vec![],
        1 => v,
        n => vec![v[0].clone(), v[n - 1].clone()],
    }
}

// ── Internal fetchers ──────────────────────────────────────────────────────────

async fn fetch_lines(fid: &str, kind: u8) -> Result<Vec<FootballLine>, String> {
    let rid = common::record_id("footballs", fid);
    let mut res = get_db()
        .query("SELECT * FROM footballs_lines WHERE football_id = $fid AND kind = $kind ORDER BY created_at ASC")
        .bind(("fid", rid))
        .bind(("kind", kind))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballLineDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(line_into).collect())
}

async fn fetch_overs(fid: &str, kind: u8) -> Result<Vec<FootballOver>, String> {
    let rid = common::record_id("footballs", fid);
    let mut res = get_db()
        .query("SELECT * FROM footballs_overs WHERE football_id = $fid AND kind = $kind ORDER BY created_at ASC")
        .bind(("fid", rid))
        .bind(("kind", kind))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballOverDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(over_into).collect())
}

// ── Enrich ─────────────────────────────────────────────────────────────────────

async fn enrich(doc: FootballDoc) -> Result<Football, String> {
    let fid = common::id_only(&doc.id);
    let lines = fetch_lines(&fid, 0).await?;
    let calcs = fetch_overs(&fid, 0).await?;
    let officials = fetch_overs(&fid, 1).await?;
    let topics = topic_db::get_topics_by_football_id(&fid).await?;
    let category = category_db::get_category_by_id(&doc.category_id).await?;

    Ok(Football {
        id: fid,
        category_id: doc.category_id,
        season: doc.season,
        home_team: doc.home_team,
        away_team: doc.away_team,
        kick_off_at_mdhm: mdhm(&doc.kick_off_at),
        kick_off_at_mdhm8: mdhm8(&doc.kick_off_at),
        created_at: common::ymdhmsz8(&doc.created_at),
        updated_at: common::ymdhmsz8(&doc.updated_at),
        hits: doc.hits.max(0) as u64,
        stars: doc.stars.max(0) as u64,
        status: doc.status,
        il_odds: il_pair(lines),
        il_calc_over: il_pair(calcs),
        football_over: officials.into_iter().last(),
        category,
        topics,
    })
}

// ── Pagination ─────────────────────────────────────────────────────────────────

// ── Public API ─────────────────────────────────────────────────────────────────

pub async fn get_footballs_in_position(
    position: &str,
    limit: i64,
) -> Result<Vec<Football>, String> {
    let tz8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let now_utc = Utc::now();
    let now8 = now_utc.with_timezone(&tz8);
    let cutoff: i64 = if now8.hour() >= 11 { 0 } else { 1 };
    let day_off = match position {
        "jt" => -cutoff,
        "zt" => -(cutoff + 1),
        _ => return Err("position must be 'jt' or 'zt'".into()),
    };

    let target = now8.date_naive() + Duration::days(day_off);
    let start_l = tz8
        .from_local_datetime(&target.and_hms_opt(11, 0, 0).unwrap())
        .single()
        .ok_or("ambiguous datetime")?;
    let end_l = start_l + Duration::days(1);

    let start_utc = start_l.with_timezone(&Utc);
    let end_utc = end_l.with_timezone(&Utc);

    let mut res = get_db()
        .query("SELECT * FROM footballs WHERE kick_off_at >= $start AND kick_off_at < $end AND status >= 0 ORDER BY kick_off_at DESC LIMIT $limit")
        .bind(("start", start_utc))
                .bind(("end", end_utc))
        .bind(("limit", limit))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for d in docs {
        out.push(enrich(d).await?);
    }
    Ok(out)
}

pub async fn get_football_by_id(id: &str) -> Result<Option<Football>, String> {
    let bare = if id.contains(':') {
        id.split(':').nth(1).unwrap_or(id)
    } else {
        id
    };
    let doc: Option<FootballDoc> = get_db()
        .select(("footballs", bare))
        .await
        .map_err(|e| e.to_string())?;
    match doc {
        Some(d) => Ok(Some(enrich(d).await?)),
        None => Ok(None),
    }
}

pub async fn get_random_football_id() -> Result<Option<String>, String> {
    let mut res = get_db()
        .query("SELECT id FROM footballs WHERE status >= 1 ORDER BY rand() LIMIT 1")
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<IdOnly> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().next().map(|d| common::id_only(&d.id)))
}

pub async fn get_footballs(
    from: i64,
    status_min: i8,
    status_max: i8,
) -> Result<FootballsResult, String> {
    let ps = constant::config().page_size;
    // Count
    let mut cres = get_db()
        .query("SELECT count() FROM footballs WHERE status >= $min AND status <= $max GROUP ALL")
        .bind(("min", status_min))
        .bind(("max", status_max))
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = cres.take(0).map_err(|e| e.to_string())?;
    let total = counts.into_iter().next().map(|c| c.count).unwrap_or(0);
    let skip = ((from - 1) * ps).max(0);

    let mut res = get_db()
        .query("SELECT * FROM footballs WHERE status >= $min AND status <= $max ORDER BY kick_off_at DESC, updated_at DESC LIMIT $ps START $skip")
        .bind(("min", status_min))
        .bind(("max", status_max))
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for d in docs {
        items.push(enrich(d).await?);
    }
    Ok(FootballsResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

pub async fn get_footballs_by_category(
    category_id: &str,
    from: i64,
) -> Result<FootballsResult, String> {
    let cid = common::record_id("categories", category_id);
    let ps = constant::config().page_size;

    let mut cres = get_db()
        .query("SELECT count() FROM footballs WHERE category_id = $cid AND status >= 1 GROUP ALL")
        .bind(("cid", cid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = cres.take(0).map_err(|e| e.to_string())?;
    let total = counts.into_iter().next().map(|c| c.count).unwrap_or(0);
    let skip = ((from - 1) * ps).max(0);

    let mut res = get_db()
        .query("SELECT * FROM footballs WHERE category_id = $cid AND status >= 1 ORDER BY kick_off_at DESC, updated_at DESC LIMIT $ps START $skip")
        .bind(("cid", cid))
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for d in docs {
        items.push(enrich(d).await?);
    }
    Ok(FootballsResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

pub async fn get_footballs_by_topic(topic_id: &str, from: i64) -> Result<FootballsResult, String> {
    let ps = constant::config().page_size;

    // Fetch distinct football_ids linked to this topic
    let q = format!(
        "SELECT VALUE football_id FROM topics_rel WHERE topic_id = '{}' AND football_id IS NOT NONE",
        topic_id
    );
    let mut rel_res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let raw_ids: Vec<String> = rel_res.take(0).map_err(|e| e.to_string())?;

    // Deduplicate
    let mut fids: Vec<String> = Vec::new();
    for id in raw_ids {
        if !fids.contains(&id) {
            fids.push(id);
        }
    }

    let total = fids.len() as u64;
    let skip = ((from - 1) * ps).max(0) as usize;
    let page_fids: Vec<String> = fids.into_iter().skip(skip).take(ps as usize).collect();

    if page_fids.is_empty() {
        return Ok(FootballsResult {
            page_info: common::make_page_info(from, ps, total),
            items: vec![],
        });
    }

    // Build IN list (ids are already record-id strings like "footballs:xxx")
    let id_list: Vec<String> = page_fids.iter().map(|id| format!("'{}'", id)).collect();
    let q = format!(
        "SELECT * FROM footballs WHERE id IN [{}] AND status >= 1 ORDER BY kick_off_at DESC",
        id_list.join(",")
    );

    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for d in docs {
        items.push(enrich(d).await?);
    }
    Ok(FootballsResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

pub async fn get_footballs_admin(from: i64) -> Result<FootballsResult, String> {
    let ps = constant::config().page_size;

    let mut cres = get_db()
        .query("SELECT count() FROM footballs GROUP ALL")
        .await
        .map_err(|e| e.to_string())?;
    let counts: Vec<CountResult> = cres.take(0).map_err(|e| e.to_string())?;
    let total = counts.into_iter().next().map(|c| c.count).unwrap_or(0);
    let skip = ((from - 1) * ps).max(0);

    let mut res = get_db()
        .query("SELECT * FROM footballs ORDER BY updated_at DESC LIMIT $ps START $skip")
        .bind(("ps", ps))
        .bind(("skip", skip))
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<FootballDoc> = res.take(0).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for d in docs {
        items.push(enrich(d).await?);
    }
    Ok(FootballsResult {
        page_info: common::make_page_info(from, ps, total),
        items,
    })
}

pub async fn update_football_status(id: &str, status: i8) -> Result<(), String> {
    let rid = common::record_id("footballs", id);
    get_db()
        .query("UPDATE $rid SET status = $status, updated_at = time::now()")
        .bind(("rid", rid))
        .bind(("status", status))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn increment_hits(id: &str) -> Result<(), String> {
    let rid = common::record_id("footballs", id);
    get_db()
        .query("UPDATE $rid SET hits += 1")
        .bind(("rid", rid))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
