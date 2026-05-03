pub mod category;
pub mod football;
pub mod topic;
pub mod user;

pub use category::Category;
pub use football::{Football, FootballLine, FootballOver, FootballsResult};
pub use topic::Topic;
pub use user::{AuthUser, User, UserSummary, UsersResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PageInfo {
    pub current_page: u32,
    pub total_pages: u32,
    pub total_count: u64,
    pub first_cursor: String,
    pub last_cursor: String,
    pub has_previous: bool,
    pub has_next: bool,
}

/// URL query params for cursor-based pagination.
/// `from=1&first=-&last=-` by default.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PageParams {
    #[serde(default = "default_from")]
    pub from: i64,
    #[serde(default = "default_cursor")]
    pub first: String,
    #[serde(default = "default_cursor")]
    pub last: String,
}

fn default_from() -> i64 {
    1
}
fn default_cursor() -> String {
    "-".to_string()
}
