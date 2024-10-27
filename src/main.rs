mod service;
use service::consume_file;
use service::read_json_config;
use service::DBManager;
use service::DBRedditComment;
use service::DBRedditSubmission;
use service::DBRedditSubmissionSmall;
use service::DBTable;

use async_channel::{Receiver, Sender};
use std::collections::HashMap;
use tokio::task;

#[tokio::main]
async fn main() {
    let config = read_json_config();

    // Create a hashmap for faster subreddit lookup
    let subreddit_list: HashMap<String, bool> = config
        .subreddit_list
        .iter()
        .map(|subreddit| (subreddit.to_lowercase(), true))
        .collect();

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
    db_mgr.check_tables::<DBRedditComment>().await;

    // Print table information
    log::info!("Table information:");
    log::info!(
        "{}: Size {}, Count {}",
        DBRedditSubmission::table_name(),
        db_mgr.get_table_size::<DBRedditSubmission>().await,
        db_mgr.get_table_count::<DBRedditSubmission>().await,
    );

    log::info!(
        "{}: Size {}, Count {}",
        DBRedditSubmissionSmall::table_name(),
        db_mgr.get_table_size::<DBRedditSubmissionSmall>().await,
        db_mgr.get_table_count::<DBRedditSubmissionSmall>().await,
    );

    // Queue to signal jobs are done
    let (sender_insert_done, receiver_insert_done): (Sender<()>, Receiver<()>) =
        async_channel::bounded(config.ingestion.qsize_jobs_done as usize);

    let sender_insert_done_clone = sender_insert_done.clone();
    let producer = task::spawn(async move {
        for folder in std::fs::read_dir(&config.target_folder).unwrap() {
            // Read recursively all files in the target folder
            let mut entries: Vec<_> = match std::fs::read_dir(folder.unwrap().path()) {
                Ok(dir) => dir
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path().to_str().unwrap().to_string())
                    .collect(),
                Err(err) => {
                    log::error!("Failed to read directory: {}", err);
                    continue;
                }
            };
            entries.sort();
            // entries.reverse();

            for filename in entries {
                consume_file(
                    &filename,
                    config.log_frequency,
                    config.parser.num_workers,
                    config.parser.qsize_to_parse,
                    subreddit_list.clone(),
                    config.ingestion.push_summarized_db,
                    db_mgr.pool.clone(),
                    sender_insert_done.clone(),
                )
                .await;
            }
        }

        drop(sender_insert_done);
    });

    let mut last_print_timestamp = std::time::Instant::now();
    let mut total_inserts = 0;
    while let Ok(_) = receiver_insert_done.recv().await {
        total_inserts += 1;

        if total_inserts % config.log_frequency != 0 {
            continue;
        }

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_print_timestamp);
        log::info!("{} items processed in {:?}", config.log_frequency, elapsed);

        last_print_timestamp = now;

        log::info!("Insert done queue size: {}", receiver_insert_done.len());
        if Sender::is_closed(&sender_insert_done_clone) {
            break;
        }
    }

    // Wait for the producer to finish
    producer.await.unwrap();
}