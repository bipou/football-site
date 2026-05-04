use serde::Deserialize;
use crate::utils::common;
use surrealdb::types::{RecordId, SurrealValue};

use crate::models::Category;
use crate::server::db::get_db;

#[derive(Debug, Deserialize, SurrealValue)]
struct CategoryDoc {
    id: RecordId,
    name: NameDoc,
    level: u8,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct NameDoc {
    zh: String,
    en: String,
}


fn into_category(d: CategoryDoc) -> Category {
    Category {
        id: common::id_only(&d.id),
        name_zh: d.name.zh,
        name_en: d.name.en,
        level: d.level,
    }
}

pub async fn get_categories() -> Result<Vec<Category>, String> {
    let mut res = get_db()
        .query("SELECT * FROM categories ORDER BY level ASC")
        .await
        .map_err(|e| e.to_string())?;
    let docs: Vec<CategoryDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_category).collect())
}

pub async fn get_category_by_id(id: &str) -> Result<Option<Category>, String> {
    let bare = if id.contains(':') {
        id.split(':').nth(1).unwrap_or(id)
    } else {
        id
    };
    let doc: Option<CategoryDoc> = get_db()
        .select(("categories", bare))
        .await
        .map_err(|e| e.to_string())?;
    Ok(doc.map(into_category))
}

pub async fn get_categories_by_levels(levels: &[u8]) -> Result<Vec<Category>, String> {
    if levels.is_empty() {
        return Ok(vec![]);
    }
    let lvls: Vec<String> = levels.iter().map(|l| l.to_string()).collect();
    let q = format!(
        "SELECT * FROM categories WHERE level IN [{}] ORDER BY level ASC",
        lvls.join(",")
    );
    let mut res = get_db().query(&q).await.map_err(|e| e.to_string())?;
    let docs: Vec<CategoryDoc> = res.take(0).map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(into_category).collect())
}
