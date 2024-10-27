use super::db_object::DBObject;
use super::raw_submission::RedditSubmission;
use super::DBTable;
use super::InsertStrategy;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct DBRedditSubmissionSmall {
    pub author: String,
    pub created_utc: DateTime<Utc>,
    pub id: String,
    pub subreddit: String,
}

impl DBTable for DBRedditSubmissionSmall {
    fn table_name() -> &'static str {
        "submissions_small"
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

impl DBObject for DBRedditSubmissionSmall {
    fn type_name() -> &'static str {
        "DBSubmissionSmall"
    }

    async fn insert(&self, pool: &PgPool, _strategy: InsertStrategy, _ignore: bool) {
        let sql = format!(
            "INSERT INTO {} (author, created_utc, id, subreddit) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
            Self::table_name()
        );

        sqlx::query(&sql)
            .bind(&self.author)
            .bind(self.created_utc)
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

impl From<&RedditSubmission> for DBRedditSubmissionSmall {
    fn from(submission: &RedditSubmission) -> Self {
        DBRedditSubmissionSmall {
            author: submission.author.clone(),
            created_utc: DateTime::<Utc>::from(&submission.created_utc),
            id: submission.id.clone(),
            subreddit: submission.subreddit.clone(),
        }
    }
}
