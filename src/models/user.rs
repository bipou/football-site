use crate::models::{PageInfo, Topic};
use serde::{Deserialize, Serialize};

/// Full user profile (used on user detail page).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub introduction: String,
    /// Markdown rendered to HTML (computed server-side)
    pub introduction_html: String,
    pub created_at: String,
    pub updated_at: String,
    /// 1-10 active, 0 inactive, -1 banned
    pub status: i8,
    pub keywords: Vec<Topic>,
    pub topics: Vec<Topic>,
}

/// Compact user info for list pages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: i8,
    pub keywords: Vec<Topic>,
    pub topics: Vec<Topic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsersResult {
    pub page_info: PageInfo,
    pub items: Vec<UserSummary>,
}

/// Authenticated user state (stored in context).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AuthUser {
    pub username: String,
    pub token: String,
}
