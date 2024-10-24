mod service;
use service::consume_file;
use service::DBManager;
use service::DBRedditSubmission;
use service::DBRedditSubmissionSmall;
use service::RedditSubmission;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio::sync::Semaphore;
use tokio::task;

#[derive(serde::Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
}

#[derive(serde::Deserialize)]
struct Config {
    target_folder: String,
    ingestion_w_summarized_db: bool,
    log_frequency: u64,
    subreddit_list: Vec<String>,
    log_file: String,
    database: DatabaseConfig,
}

fn read_json_config() -> Config {
    let config = std::fs::read_to_string("config.json").unwrap();
    let config: Config = serde_json::from_str(&config).unwrap();
    config
}

#[tokio::main]
async fn main() {
    let config = read_json_config();

    // Create a hashmap for faster subreddit lookup
    let subreddit_list: Arc<HashMap<String, bool>> = Arc::new(
        config
            .subreddit_list
            .iter()
            .map(|subreddit| (subreddit.to_lowercase(), true))
            .collect(),
    );

    // Configure logging
    simple_logging::log_to_file(config.log_file, log::LevelFilter::Info)
        .unwrap_or_else(|err| panic!("Failed to initialize logging: {}", err));
    log::info!("Starting reader");

    let db_mgr = DBManager::new(
        &config.database.host,
        config.database.port,
        &config.database.user,
        &config.database.password,
        &config.database.database,
    )
    .await;
    // Check if the tables exist, and create them if they don't
    db_mgr.check_tables::<DBRedditSubmission>().await;
    db_mgr.check_tables::<DBRedditSubmissionSmall>().await;

    // Queue to store incoming data from the file.
    let (sender, mut receiver) = channel::<RedditSubmission>(10000);

    // Read all files in the target folder
    task::spawn(async move {
        let mut entries: Vec<_> = std::fs::read_dir(config.target_folder)
            .unwrap()
            .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
            .collect();
        entries.sort();

        for filename in entries {
            consume_file(&filename, config.log_frequency, sender.clone()).await;
        }
    });

    // Limit the number of concurrent tasks
    let semaphore = Arc::new(Semaphore::new(30));
    let mut insert_count = 0;
    while let Some(json) = receiver.recv().await {
        let subreddit_list = Arc::clone(&subreddit_list);
        let pool = db_mgr.pool.clone();

        // Acquire a permit before launching a new task
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        insert_count += 1;

        // Write to log every log_frequency items
        if insert_count % config.log_frequency == 0 {
            // Get DB count and size
            let count = db_mgr.get_table_count::<DBRedditSubmission>().await;
            let size = db_mgr.get_table_size::<DBRedditSubmission>().await;
            log::info!("DB count: {}, size: {} GB", count, size);
        }

        task::spawn(async move {
            if let Err(e) = async {
                let subreddit_name = json.subreddit.clone().unwrap_or_default().to_lowercase();

                if subreddit_list.contains_key(&subreddit_name) {
                    let db_submission = DBRedditSubmission::from(json);
                    db_submission.insert(&pool).await;
                    Ok::<(), Box<dyn std::error::Error>>(())
                } else {
                    if config.ingestion_w_summarized_db {
                        let db_submission_small = DBRedditSubmissionSmall::from(json);
                        db_submission_small.insert(&pool).await;
                    }
                    Ok::<(), Box<dyn std::error::Error>>(())
                }
            }
            .await
            {
                eprintln!("Task panicked: {:?}", e);
                std::process::abort();
            }
            drop(permit);
        });
    }
}
