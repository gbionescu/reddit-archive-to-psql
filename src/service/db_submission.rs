use sqlx::PgPool;

use crate::service::raw_submission::{CreatedUTC, Edited, RedditSubmission};
use crate::service::DBTable;

pub const TABLE_SUBMISSIONS: &str = "submissions";

pub struct DBRedditSubmission {
    pub author: String,
    pub created_utc: u64,
    pub domain: String,
    pub edited: Edited,
    pub id: String,
    pub is_self: bool,
    pub num_comments: i32,
    pub permalink: String,
    pub retrieved_on: u64,
    pub score: i64,
    pub selftext: String,
    pub stickied: bool,
    pub subreddit: String,
    pub title: String,
    pub url: String,
}

impl DBRedditSubmission {
    pub async fn insert(&self, pool: &PgPool) {
        // Check if the submission already exists in the database.
        let exists = sqlx::query("SELECT id FROM submissions WHERE id = $1")
            .bind(&self.id)
            .fetch_optional(pool)
            .await
            .unwrap_or_else(|err| {
                log::error!(
                    "[DBRedditSubmission] Failed to check if submission exists: {}",
                    err
                );
                panic!("Error1");
            });

        if exists.is_some() {
            return;
        }

        let sql = format!(
            "INSERT INTO {} (author, created_utc, domain, edited, id, is_self, num_comments, permalink, \
            retrieved_on, score, selftext, stickied, subreddit, title, url) VALUES ($1, $2, $3, $4, $5, \
            $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
            TABLE_SUBMISSIONS
        );
        let timestamp_created =
            sqlx::types::chrono::DateTime::from_timestamp(self.created_utc as i64, 0);
        let timestamp_edited = match &self.edited {
            Edited::Bool(_) => None,
            Edited::Float(timestamp) => Some(sqlx::types::chrono::DateTime::from_timestamp(
                *timestamp as i64,
                0,
            )),
        };
        let timestamp_retrieved =
            sqlx::types::chrono::DateTime::from_timestamp(self.retrieved_on as i64, 0);

        sqlx::query(&sql)
            .bind(&self.author)
            .bind(timestamp_created)
            .bind(&self.domain)
            .bind(timestamp_edited)
            .bind(&self.id)
            .bind(&self.is_self)
            .bind(&self.num_comments)
            .bind(&self.permalink)
            .bind(timestamp_retrieved)
            .bind(&self.score)
            .bind(&self.selftext)
            .bind(&self.stickied)
            .bind(&self.subreddit)
            .bind(&self.title)
            .bind(&self.url)
            .execute(pool)
            .await
            .unwrap_or_else(|err| {
                log::error!("[DBRedditSubmission] Failed to insert submission: {}", err);
                panic!("Error2");
            });
    }
}

impl DBTable for DBRedditSubmission {
    fn table_name() -> &'static str {
        TABLE_SUBMISSIONS
    }

    fn sql_types() -> Vec<(&'static str, &'static str)> {
        let types = [
            ("author", "text"),
            ("created_utc", "timestamp with time zone"),
            ("domain", "text"),
            ("edited", "timestamp with time zone"),
            ("id", "text"),
            ("is_self", "boolean"),
            ("num_comments", "integer"),
            ("permalink", "text"),
            ("retrieved_on", "timestamp with time zone"),
            ("score", "bigint"),
            ("selftext", "text"),
            ("stickied", "boolean"),
            ("subreddit", "text"),
            ("subreddit_id", "text"),
            ("title", "text"),
            ("url", "text"),
        ];
        types.to_vec()
    }

    async fn post_create_table(pool: &PgPool) {
        // Make ID the primary key.
        sqlx::query(&format!(
            "ALTER TABLE {} ADD PRIMARY KEY (id)",
            TABLE_SUBMISSIONS
        ))
        .execute(pool)
        .await
        .expect("Failed to add primary key.");
    }
}

impl From<RedditSubmission> for DBRedditSubmission {
    fn from(submission: RedditSubmission) -> Self {
        let created_utc = match submission.created_utc {
            CreatedUTC::String(timestamp) => timestamp.parse::<u64>().unwrap_or_default(),
            CreatedUTC::Integer(timestamp) => timestamp as u64,
        };

        DBRedditSubmission {
            author: submission.author.unwrap_or_default(),
            created_utc: created_utc,
            domain: submission.domain,
            edited: submission.edited,
            id: submission.id,
            is_self: submission.is_self,
            num_comments: submission.num_comments,
            permalink: submission.permalink,
            retrieved_on: submission.retrieved_on.unwrap_or_default(),
            score: submission.score,
            selftext: submission.selftext,
            stickied: submission.stickied.unwrap_or_default(),
            subreddit: submission.subreddit.unwrap_or_default(),
            title: submission.title,
            url: submission.url,
        }
    }
}
