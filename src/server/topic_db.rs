use crate::utils::common::{into_rid, rid_str};
use serde::Deserialize;
use surrealdb::types::{RecordId, SurrealValue};

use crate::models::Topic;
use crate::server::db::get_db;

// ── Document types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, SurrealValue)]
struct TopicDoc {
    id: RecordId,
    name: String,
    quotes: i64,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct RelTopicId {
    topic_id: RecordId,
}

#[derive(Debug, Deserialize, SurrealValue)]
#[allow(dead_code)]
struct RelId {
    id: RecordId,
}

// ── Conversion ─────────────────────────────────────────────────────────────────

fn into_topic(d: TopicDoc) -> Topic {
    Topic {
        id: rid_str(&d.id),
        name: d.name,
        quotes: d.quotes,
    }
}

// ── Public API ─────────────────────────────────────────────────────────────────

pub async fn get_topic_by_id(rid: &RecordId) -> Result<Option<Topic>, String> {
    let doc: Option<TopicDoc> = get_db().select(rid).await.map_err(|e| e.to_string())?;
    Ok(doc.map(into_topic))
}

pub async fn get_topics_by_football_id(football_rid: &RecordId) -> Result<Vec<Topic>, String> {
    let mut res = get_db()
        .query(
            "SELECT topic_id FROM topics_rel WHERE football_id = $fid AND football_id IS NOT NONE",
        )
        .bind(("fid", football_rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    // Deduplicate topic ids
    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = rid_str(&r.topic_id);
        if !tids.contains(&tid) {
            tids.push(tid);
        }
    }

    if tids.is_empty() {
        return Ok(vec![]);
    }

    // Build IN clause with deduplicated ids
    let in_clause = tids.join(", ");
    let q = format!(
        "SELECT * FROM topics WHERE id IN [{}] ORDER BY quotes DESC",
        in_clause
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_topic).collect())
}

pub async fn get_keywords_by_user_id(user_rid: &RecordId) -> Result<Vec<Topic>, String> {
    let mut res = get_db()
        .query("SELECT topic_id FROM topics_rel WHERE user_id = $uid AND football_id IS NONE")
        .bind(("uid", user_rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = rid_str(&r.topic_id);
        if !tids.contains(&tid) {
            tids.push(tid);
        }
    }

    if tids.is_empty() {
        return Ok(vec![]);
    }

    let in_clause = tids.join(", ");
    let q = format!(
        "SELECT * FROM topics WHERE id IN [{}] ORDER BY quotes DESC",
        in_clause
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_topic).collect())
}

pub async fn get_topics_by_user_id(user_rid: &RecordId) -> Result<Vec<Topic>, String> {
    let mut res = get_db()
        .query("SELECT topic_id FROM topics_rel WHERE user_id = $uid")
        .bind(("uid", user_rid.clone()))
        .await
        .map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = rid_str(&r.topic_id);
        if !tids.contains(&tid) {
            tids.push(tid);
        }
    }

    if tids.is_empty() {
        return Ok(vec![]);
    }

    let in_clause = tids.join(", ");
    let q = format!(
        "SELECT * FROM topics WHERE id IN [{}] ORDER BY quotes DESC",
        in_clause
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_topic).collect())
}

pub async fn create_topics_from_names(names: &str) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    for raw in names.split(|c: char| c == ',' || c == ' ' || c == '\n') {
        let name = raw.trim().to_lowercase();
        if name.is_empty() {
            continue;
        }

        // Check if topic already exists
        let mut res = get_db()
            .query("SELECT * FROM topics WHERE name = $name")
            .bind(("name", name.clone()))
            .await
            .map_err(|e| e.to_string())?;
        let docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;

        if let Some(doc) = docs.first() {
            // Increment quotes on existing topic
            get_db()
                .query("UPDATE $rid SET quotes += 1")
                .bind(("rid", doc.id.clone()))
                .await
                .map_err(|e| e.to_string())?;
            ids.push(rid_str(&doc.id));
        } else {
            // Create new topic
            let mut res = get_db()
                .query("CREATE topics CONTENT { name: $name, quotes: 1 }")
                .bind(("name", name.clone()))
                .await
                .map_err(|e| e.to_string())?;
            let new_docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
            if let Some(doc) = new_docs.first() {
                ids.push(rid_str(&doc.id));
            }
        }
    }

    Ok(ids)
}

pub async fn link_topics_to_user(user_id: &str, topic_ids: Vec<String>) -> Result<(), String> {
    let user_rid = into_rid(user_id, "users");
    for tid in &topic_ids {
        let topic_rid = into_rid(tid, "topics");

        // Check if relation already exists
        let mut res = get_db()
            .query("SELECT id FROM topics_rel WHERE user_id = $uid AND topic_id = $tid AND football_id IS NONE")
            .bind(("uid", user_rid.clone()))
            .bind(("tid", topic_rid.clone()))
            .await
            .map_err(|e| e.to_string())?;
        let rels: Vec<RelId> = res.take(0).map_err(|e| e.to_string())?;

        if rels.is_empty() {
            get_db()
                .query("CREATE topics_rel CONTENT { user_id: $uid, topic_id: $tid }")
                .bind(("uid", user_rid.clone()))
                .bind(("tid", topic_rid.clone()))
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
