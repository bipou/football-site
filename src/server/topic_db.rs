use serde::Deserialize;

use crate::models::Topic;
use crate::server::db::get_db;

// ── Helpers ────────────────────────────────────────────────────────────────────

fn id_only(t: &surrealdb::sql::Thing) -> String {
    t.id.to_string()
}

fn record_id(table: &str, id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("{}:{}", table, id)
    }
}

// ── Document types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TopicDoc {
    id: surrealdb::sql::Thing,
    name: String,
    quotes: i64,
}

#[derive(Debug, Deserialize)]
struct RelTopicId {
    topic_id: surrealdb::sql::Thing,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RelId {
    id: surrealdb::sql::Thing,
}

// ── Conversion ─────────────────────────────────────────────────────────────────

fn into_topic(d: TopicDoc) -> Topic {
    Topic {
        id: id_only(&d.id),
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
    let doc: Option<TopicDoc> = get_db()
        .select(("topics", bare))
        .await
        .map_err(|e| e.to_string())?;
    Ok(doc.map(into_topic))
}

pub async fn get_topics_by_football_id(football_id: &str) -> Result<Vec<Topic>, String> {
    let fid = record_id("footballs", football_id);

    // Collect distinct topic_id values where football_id is set
    let q = format!(
        "SELECT topic_id FROM topics_relevant WHERE football_id = {} AND football_id != NONE",
        fid
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = r.topic_id.to_string();
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
    let uid = record_id("users", user_id);

    // Only topics where football_id is NONE (user's personal keywords)
    let q = format!(
        "SELECT topic_id FROM topics_relevant WHERE user_id = {} AND football_id = NONE",
        uid
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = r.topic_id.to_string();
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
    let uid = record_id("users", user_id);

    // All topics for the user (any football_id, including NONE)
    let q = format!(
        "SELECT topic_id FROM topics_relevant WHERE user_id = {}",
        uid
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let rels: Vec<RelTopicId> = res.take(0).map_err(|e| e.to_string())?;

    let mut tids: Vec<String> = Vec::new();
    for r in &rels {
        let tid = r.topic_id.to_string();
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

        let escaped = name.replace('\\', "\\\\").replace('\'', "\\'");

        // Check if topic already exists
        let select_sql = format!("SELECT * FROM topics WHERE name = '{}'", escaped);
        let mut res = get_db()
            .query(&select_sql)
            .await
            .map_err(|e| e.to_string())?;
        let docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;

        if let Some(doc) = docs.first() {
            // Increment quotes on existing topic
            let update_sql = format!("UPDATE {} SET quotes += 1", doc.id);
            get_db()
                .query(&update_sql)
                .await
                .map_err(|e| e.to_string())?;
            ids.push(id_only(&doc.id));
        } else {
            // Create new topic
            let create_sql = format!("CREATE topics CONTENT {{ name: '{}', quotes: 1 }}", escaped);
            let mut res = get_db()
                .query(&create_sql)
                .await
                .map_err(|e| e.to_string())?;
            let new_docs: Vec<TopicDoc> = res.take(0).map_err(|e| e.to_string())?;
            if let Some(doc) = new_docs.first() {
                ids.push(id_only(&doc.id));
            }
        }
    }

    Ok(ids)
}

pub async fn link_topics_to_user(user_id: &str, topic_ids: Vec<String>) -> Result<(), String> {
    let uid = record_id("users", user_id);

    for tid in &topic_ids {
        let tid_full = record_id("topics", tid);

        // Check if relation already exists (football_id = NONE for user keywords)
        let check_sql = format!(
            "SELECT id FROM topics_relevant WHERE user_id = {} AND topic_id = {} AND football_id = NONE",
            uid, tid_full
        );
        let mut res = get_db()
            .query(&check_sql)
            .await
            .map_err(|e| e.to_string())?;
        let rels: Vec<RelId> = res.take(0).map_err(|e| e.to_string())?;

        if rels.is_empty() {
            let create_sql = format!(
                "CREATE topics_relevant CONTENT {{ user_id: {}, topic_id: {} }}",
                uid, tid_full
            );
            get_db()
                .query(&create_sql)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
