use crate::utils::common;
use serde::Deserialize;
use surrealdb::types::{RecordId, SurrealValue};

use crate::models::Topic;
use crate::server::db::get_db;

// ── Helpers ────────────────────────────────────────────────────────────────────

// ── Document types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, SurrealValue)]
struct TopicDoc {
    id: RecordId,
    name: String,
    quotes: i64,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct RelTopicId {
    topic_id: String,
}

#[derive(Debug, Deserialize, SurrealValue)]
#[allow(dead_code)]
struct RelId {
    id: RecordId,
}

// ── Conversion ─────────────────────────────────────────────────────────────────

fn into_topic(d: TopicDoc) -> Topic {
    Topic {
        id: common::id_only(&d.id),
        name: d.name,
        quotes: d.quotes,
    }
}

// ── Public API ─────────────────────────────────────────────────────────────────

pub async fn get_topic_by_id(id: &str) -> Result<Option<Topic>, String> {
    let bare = if id.contains(':') {
        id.split(':').nth(1).unwrap_or(id)
    } else {
        id
    };
    let rid = RecordId::new("topics", bare);
    let doc: Option<TopicDoc> = get_db().select(rid).await.map_err(|e| e.to_string())?;
    Ok(doc.map(into_topic))
}

pub async fn get_topics_by_football_id(football_id: &str) -> Result<Vec<Topic>, String> {
    let q = format!(
        "SELECT topic_id FROM topics_rel WHERE football_id = '{}' AND football_id IS NOT NONE",
        football_id
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = format!("topics:{}", r.topic_id);
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

pub async fn get_keywords_by_user_id(user_id: &str) -> Result<Vec<Topic>, String> {
    // user_id in topics_rel may be stored as plain string (not RecordId),
    // so compare as string with quotes rather than RecordId literal.
    let q = format!(
        "SELECT topic_id FROM topics_rel WHERE user_id = '{}' AND football_id IS NONE",
        user_id
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = format!("topics:{}", r.topic_id);
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

pub async fn get_topics_by_user_id(user_id: &str) -> Result<Vec<Topic>, String> {
    // Same as get_keywords_by_user_id: use string comparison for user_id
    let q = format!(
        "SELECT topic_id FROM topics_rel WHERE user_id = '{}'",
        user_id
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = format!("topics:{}", r.topic_id);
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
            let update_sql = format!("UPDATE {} SET quotes += 1", common::rid_str(&doc.id));
            get_db()
                .query(&update_sql)
                .await
                .map_err(|e| e.to_string())?;
            ids.push(common::id_only(&doc.id));
        } else {
            // Create new topic
            let mut res = get_db()
                .query("CREATE topics CONTENT { name: $name, quotes: 1 }")
                .bind(("name", name.clone()))
                .await
                .map_err(|e| e.to_string())?;
            let new_docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
            if let Some(doc) = new_docs.first() {
                ids.push(common::id_only(&doc.id));
            }
        }
    }

    Ok(ids)
}

pub async fn link_topics_to_user(user_id: &str, topic_ids: Vec<String>) -> Result<(), String> {
    for tid in &topic_ids {
        let check_sql = format!(
            "SELECT id FROM topics_rel WHERE user_id = '{}' AND topic_id = '{}' AND football_id IS NONE",
            user_id, tid
        );
        let mut res = get_db()
            .query(&check_sql)
            .await
            .map_err(|e| e.to_string())?;
        let rels: Vec<RelId> = res.take(0).map_err(|e| e.to_string())?;

        if rels.is_empty() {
            let create_sql = format!(
                "CREATE topics_rel CONTENT {{ user_id: '{}', topic_id: '{}' }}",
                user_id, tid
            );
            get_db()
                .query(&create_sql)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
