use crate::service::db_object::DBObject;
use crate::service::raw_comment::RedditComment;
use crate::service::{DBTable, InsertStrategy};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Clone)]
pub struct DBRedditComment {
    pub author: String,
    pub body: String,
    pub created_utc: DateTime<Utc>,
    pub edited: DateTime<Utc>,
    pub id: String,
    pub link_id: String,
    pub parent_id: String,
    pub permalink: String,
    pub retrieved_on: DateTime<Utc>,
    pub score: i64,
    pub subreddit: String,
}

impl DBTable for DBRedditComment {
    fn table_name() -> &'static str {
        "comments"
    }

    fn sql_types() -> Vec<(&'static str, &'static str)> {
        vec![
            ("author", "text"),
            ("body", "text"),
            ("created_utc", "timestamp with time zone"),
            ("edited", "timestamp with time zone"),
            ("id", "text"),
            ("link_id", "text"),
            ("parent_id", "text"),
            ("permalink", "text"),
            ("retrieved_on", "timestamp with time zone"),
            ("score", "integer"),
            ("subreddit", "text"),
        ]
    }

    async fn post_create_table(pool: &PgPool) {
        sqlx::query(&format!(
            "ALTER TABLE {} ADD PRIMARY KEY (id)",
            Self::table_name()
        ))
        .execute(pool)
        .await
        .expect("Failed to add primary key.");
    }
}

impl DBObject for DBRedditComment {
    fn type_name() -> &'static str {
        "DBRedditComment"
    }

    async fn insert(&self, pool: &PgPool, insert_strategy: InsertStrategy, skip_exists: bool) {
        // Skipping existing entries if needed
        if insert_strategy == InsertStrategy::InsertUpdate && skip_exists {
            panic!("Cannot skip exists and use InsertUpdate.");
        }

        if skip_exists {
            let sql = format!("SELECT * FROM {} WHERE id = $1", Self::table_name());
            let exists = sqlx::query(&sql)
                .bind(&self.id)
                .fetch_optional(pool)
                .await
                .unwrap_or_else(|err| {
                    panic!(
                        "[DBRedditComment] Failed to check if comment exists: {}",
                        err
                    );
                });

            if exists.is_some() {
                return;
            }
        }

        let mut sql = format!(
            "INSERT INTO {} (author, body, created_utc, edited, id, link_id, parent_id, permalink, retrieved_on, score, subreddit) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            Self::table_name()
        );

        if insert_strategy == InsertStrategy::InsertIgnore {
            sql.push_str(" ON CONFLICT DO NOTHING");
        } else if insert_strategy == InsertStrategy::InsertUpdate {
            panic!("InsertUpdate not implemented.");
        } else {
            panic!("Invalid insert strategy.");
        }

        sqlx::query(&sql)
            .bind(&self.author)
            .bind(&self.body)
            .bind(self.created_utc)
            .bind(self.edited)
            .bind(&self.id)
            .bind(&self.link_id)
            .bind(&self.parent_id)
            .bind(&self.permalink)
            .bind(self.retrieved_on)
            .bind(self.score)
            .bind(&self.subreddit)
            .execute(pool)
            .await
            .unwrap_or_else(|err| {
                panic!("[DBRedditComment] Failed to insert comment: {}", err);
            });
    }
}

// Convert from RedditComment to DBRedditComment
impl From<&RedditComment> for DBRedditComment {
    fn from(comment: &RedditComment) -> Self {
        DBRedditComment {
            author: comment.author.clone(),
            body: comment.body.clone(),
            created_utc: DateTime::<Utc>::from(&comment.created_utc),
            edited: DateTime::<Utc>::from(&comment.edited),
            id: comment.id.clone(),
            link_id: comment.link_id.clone(),
            parent_id: comment
                .parent_id
                .as_ref()
                .map_or_else(String::new, |id| id.into()),
            permalink: comment.permalink.clone().unwrap_or_default(),
            retrieved_on: DateTime::<Utc>::from(&comment.retrieved_on),
            score: comment.score,
            subreddit: comment.subreddit.clone(),
        }
    }
}

// Convert from PgRow to DBRedditComment
// impl From<&PgRow> for DBRedditComment {
//     fn from(row: &PgRow) -> Self {
//         DBRedditComment {
//             author: row.get("author"),
//             body: row.get("body"),
//             created_utc: row
//                 .get::<chrono::DateTime<Utc>, _>("created_utc")
//                 .timestamp() as u64,
//             edited: row
//                 .get::<Option<chrono::DateTime<Utc>>, _>("edited")
//                 .map_or(0, |dt| dt.timestamp() as u64),
//             id: row.get("id"),
//             link_id: row.get("link_id"),
//             parent_id: row.get("parent_id"),
//             permalink: row.get("permalink"),
//             retrieved_on: row
//                 .get::<chrono::DateTime<Utc>, _>("retrieved_on")
//                 .timestamp() as u64,
//             score: row.get("score"),
//             subreddit: row.get("subreddit"),
//         }
//     }
// }
