use super::raw_object::AnyTimestamp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RedditSubmission {
    pub archived: Option<bool>,
    pub author: String,
    pub author_flair_css_class: Option<String>,
    pub author_flair_text: Option<String>,
    pub brand_safe: Option<bool>,
    pub contest_mode: Option<bool>,
    pub created_utc: AnyTimestamp,
    pub distinguished: Option<String>,
    pub domain: Option<String>,
    pub edited: AnyTimestamp,
    pub gilded: Option<i32>,
    pub hidden: Option<bool>,
    pub hide_score: Option<bool>,
    pub id: String,
    pub is_crosspostable: Option<bool>,
    pub is_reddit_media_domain: Option<bool>,
    pub is_self: bool,
    pub is_video: Option<bool>,
    pub link_flair_css_class: Option<String>,
    pub link_flair_text: Option<String>,
    pub locked: Option<bool>,
    pub media: serde_json::Value,
    pub media_embed: serde_json::Value,
    pub num_comments: i32,
    pub num_crossposts: Option<i32>,
    pub over_18: bool,
    pub parent_whitelist_status: Option<String>,
    pub permalink: String,
    pub pinned: Option<bool>,
    pub retrieved_on: AnyTimestamp,
    pub score: i64,
    pub secure_media: Option<serde_json::Value>,
    pub secure_media_embed: Option<serde_json::Value>,
    pub selftext: String,
    pub spoiler: Option<bool>,
    pub stickied: Option<bool>,
    pub subreddit: String,
    pub subreddit_id: Option<String>,
    pub subreddit_type: Option<String>,
    pub suggested_sort: Option<String>,
    pub thumbnail: Option<String>,
    pub thumbnail_height: Option<u64>,
    pub thumbnail_width: Option<u64>,
    pub title: String,
    pub url: String,
    pub whitelist_status: Option<String>,
}

impl From<&str> for RedditSubmission {
    fn from(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }
}
