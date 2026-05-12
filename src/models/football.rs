use crate::models::{Category, PageInfo, Topic};
use serde::{Deserialize, Serialize};

/// Odds record (initial or latest, kind = 0 for official aggregated lines).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FootballLine {
    pub id: String,
    pub win: String,
    pub draw: String,
    pub loss: String,
    pub kind: u8,
    pub created_at: String,
}

/// Calculated or official match result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FootballOver {
    pub id: String,
    pub s: String,   // score
    pub wdl: String, // win/draw/loss
    pub tg: String,  // total goals
    pub gd: String,  // goal diff
    pub kind: u8,
    pub created_at: String,
}

/// A football match with all resolved relations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Football {
    pub id: String,
    pub category_id: String,
    pub season: String,
    pub home_team: String,
    pub away_team: String,
    /// Formatted "MM-DD HH:MM" UTC
    pub kick_off_at_mdhm: String,
    /// Formatted "MM-DD HH:MM" UTC+8
    pub kick_off_at_mdhm8: String,
    pub created_at: String,
    pub updated_at: String,
    pub hits: u64,
    pub stars: u64,
    /// Status: 4=both,3=picks,2=hot,1=published,0=draft,-1=deleted
    pub status: i8,
    /// Up to 2 entries: [initial kind=0, latest kind=0]
    pub il_odds: Vec<FootballLine>,
    /// Up to 2 entries: [initial kind=0, latest kind=0]
    pub il_calc_over: Vec<FootballOver>,
    /// Official final result (kind=1, latest)
    pub football_over: Option<FootballOver>,
    pub category: Option<Category>,
    pub topics: Vec<Topic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FootballsResult {
    pub page_info: PageInfo,
    pub items: Vec<Football>,
}
