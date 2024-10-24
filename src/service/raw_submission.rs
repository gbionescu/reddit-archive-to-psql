use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Serialize, Debug)]
pub enum Edited {
    Float(f64),
    Bool(bool),
}

#[derive(Serialize, Debug)]
pub enum CreatedUTC {
    Integer(i64),
    String(String),
}

impl<'de> Deserialize<'de> for Edited {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(float_value) = value.as_f64() {
            Ok(Edited::Float(float_value))
        } else if let Some(bool_value) = value.as_bool() {
            Ok(Edited::Bool(bool_value))
        } else {
            Err(serde::de::Error::custom("expected float or bool"))
        }
    }
}

impl<'de> Deserialize<'de> for CreatedUTC {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(int_value) = value.as_i64() {
            Ok(CreatedUTC::Integer(int_value))
        } else if let Some(string_value) = value.as_str() {
            Ok(CreatedUTC::String(string_value.to_string()))
        } else {
            Err(serde::de::Error::custom("expected integer or string"))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RedditSubmission {
    pub archived: Option<bool>,
    pub author: Option<String>,
    pub author_flair_css_class: Option<String>,
    pub author_flair_text: Option<String>,
    pub brand_safe: Option<bool>,
    pub contest_mode: Option<bool>,
    pub created_utc: CreatedUTC,
    pub distinguished: Option<String>,
    pub domain: String,
    pub edited: Edited,
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
    pub retrieved_on: Option<u64>,
    pub score: i64,
    pub secure_media: Option<serde_json::Value>,
    pub secure_media_embed: Option<serde_json::Value>,
    pub selftext: String,
    pub spoiler: Option<bool>,
    pub stickied: Option<bool>,
    pub subreddit: Option<String>,
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
