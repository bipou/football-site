use bson::{doc, oid::ObjectId};
use mongodb::Collection;
use serde::Deserialize;

use crate::models::Topic;
use crate::server::db::get_db;

#[derive(Debug, Deserialize)]
struct TopicDoc {
    #[serde(rename = "_id")]
    id: ObjectId,
    name: String,
    quotes: i64,
}

fn into_topic(d: TopicDoc) -> Topic {
    Topic { id: d.id.to_hex(), name: d.name, quotes: d.quotes }
}

pub async fn get_topic_by_id(id: &str) -> Result<Option<Topic>, String> {
    let oid = ObjectId::parse_str(id).map_err(|e| e.to_string())?;
    let coll: Collection<TopicDoc> = get_db().collection("topics");
    Ok(coll.find_one(doc! { "_id": oid }).await.map_err(|e| e.to_string())?.map(into_topic))
}

pub async fn get_topics_by_football_id(football_id: &str) -> Result<Vec<Topic>, String> {
    let fid = ObjectId::parse_str(football_id).map_err(|e| e.to_string())?;
    let db  = get_db();

    #[derive(Deserialize)]
    struct Rel { topic_id: ObjectId }
    let rel: Collection<Rel> = db.collection("topics_relevant");
    let mut cur = rel.find(doc! { "football_id": fid }).await.map_err(|e| e.to_string())?;
    let mut tids: Vec<ObjectId> = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        let r = cur.deserialize_current().map_err(|e| e.to_string())?;
        if !tids.contains(&r.topic_id) { tids.push(r.topic_id); }
    }
    if tids.is_empty() { return Ok(vec![]); }

    let coll: Collection<TopicDoc> = db.collection("topics");
    let mut cur = coll.find(doc! { "_id": { "$in": &tids } }).sort(doc! { "quotes": -1 }).await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        out.push(into_topic(cur.deserialize_current().map_err(|e| e.to_string())?));
    }
    Ok(out)
}

pub async fn get_keywords_by_user_id(user_id: &str) -> Result<Vec<Topic>, String> {
    let uid = ObjectId::parse_str(user_id).map_err(|e| e.to_string())?;
    let db  = get_db();

    #[derive(Deserialize)]
    struct Rel { topic_id: ObjectId, #[serde(default)] football_id: Option<ObjectId> }
    let rel: Collection<Rel> = db.collection("topics_relevant");
    let mut cur = rel.find(doc! { "user_id": uid, "football_id": { "$exists": false } }).await.map_err(|e| e.to_string())?;
    let mut tids: Vec<ObjectId> = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        let r = cur.deserialize_current().map_err(|e| e.to_string())?;
        if r.football_id.is_none() && !tids.contains(&r.topic_id) { tids.push(r.topic_id); }
    }
    if tids.is_empty() { return Ok(vec![]); }

    let coll: Collection<TopicDoc> = db.collection("topics");
    let mut cur = coll.find(doc! { "_id": { "$in": &tids } }).sort(doc! { "quotes": -1 }).await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        out.push(into_topic(cur.deserialize_current().map_err(|e| e.to_string())?));
    }
    Ok(out)
}

pub async fn get_topics_by_user_id(user_id: &str) -> Result<Vec<Topic>, String> {
    let uid = ObjectId::parse_str(user_id).map_err(|e| e.to_string())?;
    let db  = get_db();

    #[derive(Deserialize)]
    struct Rel { topic_id: ObjectId }
    let rel: Collection<Rel> = db.collection("topics_relevant");
    let mut cur = rel.find(doc! { "user_id": uid }).await.map_err(|e| e.to_string())?;
    let mut tids: Vec<ObjectId> = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        let r = cur.deserialize_current().map_err(|e| e.to_string())?;
        if !tids.contains(&r.topic_id) { tids.push(r.topic_id); }
    }
    if tids.is_empty() { return Ok(vec![]); }

    let coll: Collection<TopicDoc> = db.collection("topics");
    let mut cur = coll.find(doc! { "_id": { "$in": &tids } }).sort(doc! { "quotes": -1 }).await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        out.push(into_topic(cur.deserialize_current().map_err(|e| e.to_string())?));
    }
    Ok(out)
}

pub async fn create_topics_from_names(names: &str) -> Result<Vec<ObjectId>, String> {
    let db   = get_db();
    let coll: Collection<TopicDoc> = db.collection("topics");
    let mut ids = Vec::new();
    for raw in names.split(|c: char| c == ',' || c == ' ' || c == '\n') {
        let name = raw.trim().to_lowercase();
        if name.is_empty() { continue; }
        // Increment quotes if exists, otherwise insert with quotes=1
        let doc = coll
            .find_one_and_update(doc! { "name": &name }, doc! { "$inc": { "quotes": 1i64 } })
            .upsert(true)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(d) = doc {
            ids.push(d.id);
        } else if let Some(d) = coll.find_one(doc! { "name": &name }).await.map_err(|e| e.to_string())? {
            ids.push(d.id);
        }
    }
    Ok(ids)
}

pub async fn link_topics_to_user(user_id: ObjectId, topic_ids: Vec<ObjectId>) -> Result<(), String> {
    let coll = get_db().collection::<bson::Document>("topics_relevant");
    for tid in topic_ids {
        let exists = coll
            .find_one(doc! { "user_id": user_id, "topic_id": tid, "football_id": { "$exists": false } })
            .await
            .map_err(|e| e.to_string())?;
        if exists.is_none() {
            coll.insert_one(doc! { "user_id": user_id, "topic_id": tid })
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
