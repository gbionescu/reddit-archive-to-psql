use serde_json;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::service::helpers::{debug_submission, TotalProgress};
use crate::service::raw_submission::RedditSubmission;

pub async fn consume_file(fname: &str, log_frequency: u64, channel: Sender<RedditSubmission>) {
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

        println!("child status was: {}", status);
    });

    let mut line_count = 0;
    while let Ok(Some(line)) = reader.next_line().await {
        line_count += 1;

        if line_count < fprogress.total_lines_file(fname) {
            continue;
        }

        // Remove leading \0 characters.
        let line = line.trim_start_matches(char::from(0));

        // deserialize the line into a JSON object
        let data = serde_json::from_str(&line);
        let json: RedditSubmission = match data {
            Ok(v) => v,
            Err(_e) => {
                debug_submission(&line);
                panic!("Error");
            }
        };

        fprogress.update_file(fname, &line);

        match channel.send(json).await {
            Ok(_) => {}
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    fprogress.finish_file(fname);
}
