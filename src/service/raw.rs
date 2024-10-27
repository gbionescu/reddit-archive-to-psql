use crate::service::db_object::DBObject;
use crate::service::helpers::TotalProgress;
use crate::service::InsertStrategy;
use async_channel::{Receiver, Sender};
use sqlx::pool;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::raw_comment::RedditComment;
use super::{DBRedditComment, DBRedditSubmission, RedditSubmission};

async fn sub_consume_line(
    line: &str,
    subreddit_list: &HashMap<String, bool>,
    ingestion_w_summarized_db: bool,
    pool: &pool::Pool<sqlx::Postgres>,
    channel: &Sender<()>,
) {
    // Remove leading \0 characters.
    let line = line.trim_start_matches(char::from(0));

    // deserialize the line into a JSON object
    let json = RedditSubmission::from(line);

    let subreddit_name = json.subreddit.to_lowercase();
    if subreddit_list.contains_key(&subreddit_name) {
        let db_submission = DBRedditSubmission::from(&json);
        db_submission
            .insert(&pool, InsertStrategy::InsertIgnore, true)
            .await;
    }

    if ingestion_w_summarized_db {
        // let db_submission_small = ObjSmallTable::from(&json);
        // db_submission_small
        //     .insert(&pool, InsertStrategy::InsertIgnore, true)
        //     .await;
    }

    match channel.send(()).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

async fn com_consume_line(
    line: &str,
    subreddit_list: &HashMap<String, bool>,
    ingestion_w_summarized_db: bool,
    pool: &pool::Pool<sqlx::Postgres>,
    channel: &Sender<()>,
) {
    // Remove leading \0 characters.
    let line = line.trim_start_matches(char::from(0));

    // deserialize the line into a JSON object
    let json = RedditComment::from(line);

    let subreddit_name = json.subreddit.to_lowercase();

    if subreddit_list.contains_key(&subreddit_name) {
        let db_submission = DBRedditComment::from(&json);
        db_submission
            .insert(&pool, InsertStrategy::InsertIgnore, true)
            .await;
    }

    if ingestion_w_summarized_db {
        // let db_submission_small = ObjSmallTable::from(&json);
        // db_submission_small
        //     .insert(&pool, InsertStrategy::InsertIgnore, true)
        //     .await;
    }

    match channel.send(()).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub async fn consume_file(
    fname: &str,
    log_frequency: u64,
    num_workers: u64,
    qsize_to_parse: u64,
    subreddit_list: HashMap<String, bool>,
    ingestion_w_summarized_db: bool,
    pool: pool::Pool<sqlx::Postgres>,
    channel: Sender<()>,
) {
    log::info!("Processing file: {}", fname);

    // Create a progress tracker for the file.
    let mut fprogress: TotalProgress = TotalProgress::new(log_frequency);
    fprogress.add_file(fname);

    // Check if the file has already been processed.
    if fprogress.is_file_done(fname) {
        log::info!("File already processed: {}", fname);
        return;
    }

    // Create a pipe to read the decompressed data.
    let mut cmd = Command::new("zstd");
    cmd.arg("--memory=2048MB").arg("-d").arg("-c").arg(fname);
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn zstd process");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async move {
        let status = child
            .wait()
            .await
            .expect("child process encountered an error");

        log::info!("child status was: {}", status);
    });

    let (sender_line, receiver_line): (Sender<String>, Receiver<String>) =
        async_channel::bounded(qsize_to_parse as usize);

    for _ in 0..num_workers {
        let receiver_line = receiver_line.clone();
        let channel = channel.clone();
        let subreddit_list = subreddit_list.clone();
        let pool = pool.clone();

        if fname.contains("RS_") {
            tokio::spawn(async move {
                while let Ok(line) = receiver_line.recv().await {
                    sub_consume_line(
                        &line,
                        &subreddit_list,
                        ingestion_w_summarized_db,
                        &pool,
                        &channel,
                    )
                    .await;
                }

                log::info!("Subreddit consumer done");
            });
        } else {
            tokio::spawn(async move {
                while let Ok(line) = receiver_line.recv().await {
                    com_consume_line(
                        &line,
                        &subreddit_list,
                        ingestion_w_summarized_db,
                        &pool,
                        &channel,
                    )
                    .await;
                }

                log::info!("Comment consumer done");
            });
        }
    }

    let mut line_count = 0;
    while let Ok(Some(line)) = reader.next_line().await {
        line_count += 1;

        if line_count < fprogress.total_lines_file(fname) {
            continue;
        }
        sender_line.send(line.clone()).await.unwrap();

        if fprogress.update_file(fname, &line) {
            log::info!("Queue sender {}", sender_line.len(),);
        }
    }

    fprogress.finish_file(fname);

    // Wait for all the consumers to finish.
    while !receiver_line.is_empty() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    drop(sender_line);
    drop(receiver_line);
}
