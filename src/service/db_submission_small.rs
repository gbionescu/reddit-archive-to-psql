use sqlx::PgPool;

use crate::service::raw_submission::{CreatedUTC, RedditSubmission};
use crate::service::DBTable;

pub const TABLE_SUBMISSIONS_SMALL: &str = "submissions_small";

pub struct DBRedditSubmissionSmall {
    pub author: String,
    pub created_utc: u64,
    pub id: String,
    pub subreddit: String,
}

impl DBRedditSubmissionSmall {
    pub async fn insert(&self, pool: &PgPool) {
        // Check if the submission already exists in the database.
        let exists = sqlx::query("SELECT id FROM submissions WHERE id = $1")
            .bind(&self.id)
            .fetch_optional(pool)
            .await
            .unwrap_or_else(|err| {
                log::error!(
                    "[DBRedditSubmissionSmall] Failed to check if submission exists: {}",
                    err
                );
                panic!("Error1");
            });

        if exists.is_some() {
            return;
        }

        let sql = format!(
            "INSERT INTO {} (author, created_utc, id, subreddit) VALUES ($1, $2, $3, $4)",
            TABLE_SUBMISSIONS_SMALL
        );
        let timestamp_created =
            sqlx::types::chrono::DateTime::from_timestamp(self.created_utc as i64, 0);

        sqlx::query(&sql)
            .bind(&self.author)
            .bind(timestamp_created)
            .bind(&self.id)
            .bind(&self.subreddit)
            .execute(pool)
            .await
            .unwrap_or_else(|err| {
                log::error!(
                    "[DBRedditSubmissionSmall] Failed to insert submission: {}",
                    err
                );
                panic!("Error2");
            });
    }
}

impl DBTable for DBRedditSubmissionSmall {
    fn table_name() -> &'static str {
        TABLE_SUBMISSIONS_SMALL
    }

    fn sql_types() -> Vec<(&'static str, &'static str)> {
        let types = [
            ("author", "text"),
            ("created_utc", "timestamp with time zone"),
            ("id", "text"),
            ("subreddit", "text"),
        ];
        types.to_vec()
    }

    async fn post_create_table(_pool: &PgPool) {}
}

impl From<RedditSubmission> for DBRedditSubmissionSmall {
    fn from(submission: RedditSubmission) -> Self {
        let created_utc = match submission.created_utc {
            CreatedUTC::String(timestamp) => timestamp.parse::<u64>().unwrap_or_default(),
            CreatedUTC::Integer(timestamp) => timestamp as u64,
        };

        DBRedditSubmissionSmall {
            author: submission.author.unwrap_or_default(),
            created_utc: created_utc,
            id: submission.id,
            subreddit: submission.subreddit.unwrap_or_default(),
        }
    }
}
