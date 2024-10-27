use serde::{Deserialize, Deserializer};

use super::helpers::debug_comment;
use super::raw_object::AnyTimestamp;

#[derive(Debug)]
pub enum ParentId {
    String(String),
    Int(i64),
}

impl<'de> Deserialize<'de> for ParentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(string_value) => Ok(ParentId::String(string_value)),
            serde_json::Value::Number(int_value) => {
                let int_value = int_value.as_i64().unwrap();
                Ok(ParentId::Int(int_value))
            }
            _ => panic!("expected string or int, found {:?}", value),
        }
    }
}

impl From<&ParentId> for String {
    fn from(parent_id: &ParentId) -> Self {
        match parent_id {
            ParentId::String(string_value) => string_value.clone(),
            ParentId::Int(int_value) => format!("unknown-{}", int_value),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
pub struct RedditComment {
    pub _meta: Option<serde_json::Value>,
    pub all_awardings: Option<Vec<serde_json::Value>>,
    pub approved_at_utc: Option<u64>,
    pub approved_by: Option<String>,
    pub archived: Option<bool>,
    pub associated_award: Option<serde_json::Value>,
    pub author: String,
    pub author_cakeday: Option<bool>,
    pub author_flair_background_color: Option<String>,
    pub author_flair_css_class: Option<String>,
    pub author_flair_richtext: Option<Vec<serde_json::Value>>,
    pub author_flair_template_id: Option<String>,
    pub author_flair_text: Option<String>,
    pub author_flair_text_color: Option<String>,
    pub author_flair_type: Option<String>,
    pub author_fullname: Option<String>,
    pub author_is_blocked: Option<bool>,
    pub author_patreon_flair: Option<bool>,
    pub author_premium: Option<bool>,
    pub awarders: Option<Vec<serde_json::Value>>,
    pub banned_at_utc: Option<u64>,
    pub banned_by: Option<String>,
    pub body: String,
    pub body_sha1: Option<String>,
    pub can_gild: Option<bool>,
    pub can_mod_post: Option<bool>,
    pub collapsed: Option<bool>,
    pub collapsed_because_crowd_control: Option<serde_json::Value>,
    pub collapsed_reason: Option<String>,
    pub collapsed_reason_code: Option<String>,
    pub comment_type: Option<String>,
    pub controversiality: Option<i64>,
    pub created: Option<AnyTimestamp>,
    pub created_utc: AnyTimestamp,
    pub distinguished: Option<String>,
    pub downs: Option<i64>,
    pub editable: Option<bool>,
    pub edited: AnyTimestamp,
    pub edited_on: Option<AnyTimestamp>,
    pub expression_asset_data: Option<serde_json::Value>,
    pub gilded: i64,
    pub gildings: Option<serde_json::Value>,
    pub id: String,
    pub is_submitter: Option<bool>,
    pub likes: Option<serde_json::Value>,
    pub link_id: String,
    pub locked: Option<bool>,
    pub nest_level: Option<i64>,
    pub media_metadata: Option<serde_json::Value>,
    pub mod_note: Option<String>,
    pub mod_reason_by: Option<String>,
    pub mod_reason_title: Option<String>,
    pub mod_reports: Option<Vec<serde_json::Value>>,
    pub name: Option<String>,
    pub no_follow: Option<bool>,
    pub num_reports: Option<serde_json::Value>,
    pub parent_id: Option<ParentId>,
    pub permalink: Option<String>,
    pub removal_reason: Option<String>,
    pub replies: Option<serde_json::Value>,
    pub report_reasons: Option<serde_json::Value>,
    pub retrieved_on: AnyTimestamp,
    pub rte_mode: Option<String>,
    pub saved: Option<bool>,
    pub score: i64,
    pub score_hidden: Option<bool>,
    pub send_replies: Option<bool>,
    pub stickied: Option<bool>,
    pub subreddit: String,
    pub subreddit_id: String,
    pub subreddit_name_prefixed: Option<String>,
    pub subreddit_type: Option<String>,
    pub top_awarded_type: Option<serde_json::Value>,
    pub total_awards_received: Option<i64>,
    pub treatment_tags: Option<Vec<serde_json::Value>>,
    pub unrepliable_reason: Option<String>,
    pub updated_on: Option<AnyTimestamp>,
    pub ups: Option<i64>,
    pub user_reports: Option<Vec<serde_json::Value>>,
    pub user_reports_dismissed: Option<serde_json::Value>,
}

impl From<&str> for RedditComment {
    fn from(s: &str) -> Self {
        let result_or_err = serde_json::from_str(s);

        // match result or error
        let data: RedditComment = match result_or_err {
            Ok(v) => v,
            Err(_e) => {
                debug_comment(s);
                panic!("Error decoding RedditComment")
            }
        };
        data
    }
}
