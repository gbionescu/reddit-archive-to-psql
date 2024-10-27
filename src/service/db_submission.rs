use crate::service::db_object::DBObject;
use crate::service::raw_submission::RedditSubmission;
use crate::service::{DBTable, InsertStrategy};
use sqlx::types::chrono::{DateTime, Utc};

use sqlx::PgPool;

pub const TABLE_SUBMISSIONS: &str = "submissions";

#[derive(Clone)]
pub struct DBRedditSubmission {
    pub author: String,
    pub created_utc: DateTime<Utc>,
    pub domain: String,
    pub edited: DateTime<Utc>,
    pub id: String,
    pub is_self: bool,
    pub num_comments: i32,
    pub permalink: String,
    pub retrieved_on: DateTime<Utc>,
    pub score: i64,
    pub selftext: String,
    pub stickied: bool,
    pub subreddit: String,
    pub title: String,
    pub url: String,
}

impl DBRedditSubmission {
    // pub async fn get_broken_date(pool: &PgPool) -> Vec<DBRedditSubmission> {
    //     let sql = "SELECT * FROM submissions WHERE date_trunc('year', created_utc) < '1990-01-01 00:00:00'";
    //     let broken_dates = sqlx::query(&sql)
    //         .fetch_all(pool)
    //         .await
    //         .unwrap_or_else(|err| {
    //             log::error!("[DBRedditSubmission] Failed to get broken dates: {}", err);
    //             panic!("Error3");
    //         });

    //     broken_dates
    //         .into_iter()
    //         .map(|row| DBRedditSubmission::from(&row))
    //         .collect::<Vec<DBRedditSubmission>>()
    // }
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

impl DBObject for DBRedditSubmission {
    fn type_name() -> &'static str {
        "DBRedditSubmission"
    }

    async fn insert(&self, pool: &PgPool, insert_strategy: InsertStrategy, skip_exists: bool) {
        // Skip exists and InsertUpdate are mutually exclusive.
        if insert_strategy == InsertStrategy::InsertUpdate && skip_exists {
            panic!("Cannot skip exists and use InsertUpdate.");
        }

        if skip_exists {
            let sql = format!("SELECT * FROM {} WHERE id = $1", TABLE_SUBMISSIONS);
            let exists = sqlx::query(&sql)
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
        }

        let sql;
        if insert_strategy == InsertStrategy::InsertUpdate {
            sql = format!(
                "INSERT INTO {} (author, created_utc, domain, edited, id, is_self, num_comments, permalink, \
                retrieved_on, score, selftext, stickied, subreddit, title, url) VALUES ($1, $2, $3, $4, $5, \
                $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) ON CONFLICT (id) DO UPDATE SET \
                author = $1, created_utc = $2, domain = $3, edited = $4, is_self = $6, num_comments = $7, \
                permalink = $8, retrieved_on = $9, score = $10, selftext = $11, stickied = $12, subreddit = $13, \
                title = $14, url = $15",

                TABLE_SUBMISSIONS
            );
        } else if insert_strategy == InsertStrategy::InsertIgnore {
            sql = format!(
                "INSERT INTO {} (author, created_utc, domain, edited, id, is_self, num_comments, permalink, \
                retrieved_on, score, selftext, stickied, subreddit, title, url) VALUES ($1, $2, $3, $4, $5, \
                $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) ON CONFLICT DO NOTHING",

                TABLE_SUBMISSIONS
            );
        } else {
            panic!("Invalid insert strategy.");
        }

        sqlx::query(&sql)
            .bind(&self.author)
            .bind(self.created_utc)
            .bind(&self.domain)
            .bind(self.edited)
            .bind(&self.id)
            .bind(&self.is_self)
            .bind(&self.num_comments)
            .bind(&self.permalink)
            .bind(self.retrieved_on)
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

impl From<&RedditSubmission> for DBRedditSubmission {
    fn from(submission: &RedditSubmission) -> Self {
        DBRedditSubmission {
            author: submission.author.clone(),
            created_utc: DateTime::<Utc>::from(&submission.created_utc),
            domain: submission.domain.clone().unwrap_or_default(),
            edited: DateTime::<Utc>::from(&submission.edited),
            id: submission.id.clone(),
            is_self: submission.is_self,
            num_comments: submission.num_comments,
            permalink: submission.permalink.clone(),
            retrieved_on: DateTime::<Utc>::from(&submission.retrieved_on),
            score: submission.score,
            selftext: submission.selftext.clone(),
            stickied: submission.stickied.unwrap_or_default(),
            subreddit: submission.subreddit.clone(),
            title: submission.title.clone(),
            url: submission.url.clone(),
        }
    }
}

// impl From<&PgRow> for DBRedditSubmission {
//     fn from(row: &PgRow) -> Self {
//         DBRedditSubmission {
//             author: row.get("author"),
//             created_utc: row
//                 .get::<chrono::DateTime<chrono::Utc>, _>("created_utc")
//                 .timestamp() as u64,
//             domain: row.get("domain"),
//             edited: row
//                 .get::<Option<chrono::DateTime<chrono::Utc>>, _>("edited")
//                 .map_or(0, |dt| dt.timestamp() as u64),
//             id: row.get("id"),
//             is_self: row.get("is_self"),
//             num_comments: row.get("num_comments"),
//             permalink: row.get("permalink"),
//             retrieved_on: row
//                 .get::<chrono::DateTime<chrono::Utc>, _>("retrieved_on")
//                 .timestamp() as u64,
//             score: row.get("score"),
//             selftext: row.get("selftext"),
//             stickied: row.get("stickied"),
//             subreddit: row.get("subreddit"),
//             title: row.get("title"),
//             url: row.get("url"),
//         }
//     }
// }
