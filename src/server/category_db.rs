use bson::{doc, oid::ObjectId};
use mongodb::Collection;
use serde::Deserialize;

use crate::models::Category;
use crate::server::db::get_db;

#[derive(Debug, Deserialize)]
struct CategoryDoc {
    #[serde(rename = "_id")]
    id: ObjectId,
    name: NameDoc,
    level: u8,
}

#[derive(Debug, Deserialize)]
struct NameDoc {
    zh: String,
    en: String,
}

fn into_category(d: CategoryDoc) -> Category {
    Category { id: d.id.to_hex(), name_zh: d.name.zh, name_en: d.name.en, level: d.level }
}

pub async fn get_categories() -> Result<Vec<Category>, String> {
    let coll: Collection<CategoryDoc> = get_db().collection("categories");
    let mut cur = coll.find(doc! {}).sort(doc! { "level": 1 }).await.map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        out.push(into_category(cur.deserialize_current().map_err(|e| e.to_string())?));
    }
    Ok(out)
}

pub async fn get_category_by_id(id: &str) -> Result<Option<Category>, String> {
    let oid = ObjectId::parse_str(id).map_err(|e| e.to_string())?;
    let coll: Collection<CategoryDoc> = get_db().collection("categories");
    Ok(coll.find_one(doc! { "_id": oid }).await.map_err(|e| e.to_string())?.map(into_category))
}

pub async fn get_categories_by_levels(levels: &[u8]) -> Result<Vec<Category>, String> {
    let bson_lvls: Vec<bson::Bson> = levels.iter().map(|l| bson::Bson::Int32(*l as i32)).collect();
    let coll: Collection<CategoryDoc> = get_db().collection("categories");
    let mut cur = coll
        .find(doc! { "level": { "$in": bson_lvls } })
        .sort(doc! { "level": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cur.advance().await.map_err(|e| e.to_string())? {
        out.push(into_category(cur.deserialize_current().map_err(|e| e.to_string())?));
    }
    Ok(out)
}
