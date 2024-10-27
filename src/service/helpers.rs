use crate::service::raw_comment::RedditComment;
use crate::service::raw_submission::RedditSubmission;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const TOTAL_PROGRESS_FILE: &str = "total.json";

#[derive(Deserialize, Serialize)]
/// Represents the total progress of multiple files.
pub struct TotalProgress {
    files: HashMap<String, FileProgress>,

    #[serde(skip)]
    log_frequency: u64,
}

#[derive(Deserialize, Serialize, Debug)]
/// Represents the progress of reading a file.
struct FileProgress {
    total_lines: u64,
    total_read: u64,
    is_done: bool,
    fname: String,
}

impl FileProgress {
    fn new(fname: &str) -> FileProgress {
        FileProgress {
            total_lines: 0,
            total_read: 0,
            is_done: false,
            fname: fname.to_string(),
        }
    }
}

impl TotalProgress {
    pub fn new(log_frequency: u64) -> TotalProgress {
        if std::fs::metadata(TOTAL_PROGRESS_FILE).is_ok() {
            let contents = std::fs::read_to_string(TOTAL_PROGRESS_FILE).unwrap();
            let mut data: TotalProgress = serde_json::from_str(&contents).unwrap();

            data.log_frequency = log_frequency;

            return data;
        } else {
            TotalProgress {
                files: HashMap::<String, FileProgress>::new(),
                log_frequency,
            }
        }
    }

    /// Save the total progress to a file.
    fn save_to_file(&self) {
        let contents = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(TOTAL_PROGRESS_FILE, contents).unwrap();
    }

    /// Add a file to the total progress.
    pub fn add_file(&mut self, fname: &str) {
        if self.files.contains_key(fname) {
            return;
        }

        self.files
            .insert(fname.to_string(), FileProgress::new(fname));
    }

    /// Update the progress of a file.
    pub fn update_file(&mut self, fname: &str, line: &str) -> bool {
        let file = self.files.get_mut(fname).unwrap();

        file.total_lines += 1;
        file.total_read += line.len() as u64;
        if file.total_lines % self.log_frequency == 0 {
            log::info!(
                "[{}] Processed {} lines, {} GB",
                fname,
                file.total_lines,
                bytes_to_gb(file.total_read)
            );

            self.save_to_file();
            return true;
        }

        false
    }

    /// Finish processing a file.
    pub fn finish_file(&mut self, fname: &str) {
        let file = self.files.get_mut(fname).unwrap();
        file.is_done = true;
        log::info!(
            "[{}] Finished processing {} lines, {} GB",
            fname,
            file.total_lines,
            bytes_to_gb(file.total_read)
        );

        self.save_to_file();
    }

    /// Get the total lines read for a file.
    pub fn total_lines_file(&self, fname: &str) -> u64 {
        let file = self.files.get(fname).unwrap();
        file.total_lines
    }

    /// Check if a file is done.
    pub fn is_file_done(&self, fname: &str) -> bool {
        let file = self.files.get(fname).unwrap();
        file.is_done
    }
}

// Convert bytes to gigabytes.
pub fn bytes_to_gb(bytes: u64) -> f64 {
    let converted = bytes as f64 / 1024.0 / 1024.0 / 1024.0;

    // Keep only 2 decimal places.
    (converted * 100.0).round() / 100.0
}

// Debug a submission line.
pub fn debug_submission(line: &str) {
    let data: Value = match serde_json::from_str(&line) {
        Ok(v) => v,
        Err(e) => {
            panic!("Error: {} for {}", e, line);
        }
    };

    let pretty_json = serde_json::to_string_pretty(&data).unwrap();

    // Try again to deserialize the JSON object.
    let retry: Result<RedditSubmission, serde_json::Error> = serde_json::from_str(&pretty_json);
    match retry {
        Ok(v) => v,
        Err(e) => {
            let error_line = e.line();

            // For each line in pretty_json print the line number and the line.
            for (mut i, line) in pretty_json.lines().enumerate() {
                i = i + 1;
                let i = i as u64;
                let error_line = error_line as u64;

                if i < error_line - 3 || i > error_line + 3 {
                    continue;
                }

                if i < error_line {
                    println!("-{}: {}", i, line);
                } else if i == error_line {
                    println!(">{}: {}", i, line);
                } else {
                    println!("+{}: {}", i, line);
                }
            }

            panic!("Error: {}", e);
        }
    };
}

pub fn debug_comment(line: &str) {
    let data: Value = match serde_json::from_str(&line) {
        Ok(v) => v,
        Err(e) => {
            panic!("Error: {} for {}", e, line);
        }
    };

    let pretty_json = serde_json::to_string_pretty(&data).unwrap();

    // Try again to deserialize the JSON object.
    let retry: Result<RedditComment, serde_json::Error> = serde_json::from_str(&pretty_json);
    match retry {
        Ok(v) => v,
        Err(e) => {
            let error_line = e.line();

            // For each line in pretty_json print the line number and the line.
            for (mut i, line) in pretty_json.lines().enumerate() {
                i = i + 1;

                let i = i as i64;
                let error_line = error_line as i64;

                // if i < error_line - 3 || i > error_line + 3 {
                //     continue;
                // }

                if i < error_line {
                    log::error!("-{}: {}", i, line);
                } else if i == error_line {
                    log::error!(">{}: {}", i, line);
                } else {
                    log::error!("+{}: {}", i, line);
                }
            }

            log::error!("Error: {}", e);
            panic!("Error: {}", e);
            // Kill the process.
        }
    };
}
