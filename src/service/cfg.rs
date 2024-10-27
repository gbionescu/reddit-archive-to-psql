use serde_json;

#[derive(serde::Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(serde::Deserialize)]
pub struct Ingestion {
    pub push_summarized_db: bool,
    pub qsize_jobs_done: u64,
}

#[derive(serde::Deserialize)]
pub struct Parser {
    pub num_workers: u64,
    pub qsize_to_parse: u64,
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub target_folder: String,
    pub log_frequency: u64,
    pub subreddit_list: Vec<String>,
    pub log_file: String,
    pub database: DatabaseConfig,
    pub ingestion: Ingestion,
    pub parser: Parser,
}

pub fn read_json_config() -> Config {
    let config = std::fs::read_to_string("config.json").unwrap();
    let config: Config = serde_json::from_str(&config).unwrap();
    config
}
