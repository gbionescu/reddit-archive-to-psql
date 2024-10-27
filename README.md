# Reddit archive to PSQL

Import Reddit archive data into a PostgreSQL database.

Based on Academic Torrents Reddit data dump [here](https://academictorrents.com/details/9c263fc85366c1ef8f5bb9da0203f4c8c8db75f4).

## Running the project

1. Download the Reddit data dump from the link above.
2. Build the project.
3. Edit the config file `config.json`:
    - `target_folder`: the folder where the Reddit data dump is located.
    - `subreddit_list`: a list of subreddits to import.
    - "database":
        - `host`: the host of the PostgreSQL database.
        - `port`: the port of the PostgreSQL database.
        - `database`: the name of the PostgreSQL database.
        - `user`: the user of the PostgreSQL database.
        - `password`: the password of the PostgreSQL database.
    - `ingestion`:
        - `push_summarized_db": Create a summarized database with the a reduced number of columns.
        - `qsize_jobs_done": Queue size for the jobs done queue. Can be used to throttle the ingestion.
    - `parser`:
        - `num_workers`: the number of workers to use for parsing and pushing to the database.
        - `qsize_to_parse`: Queue size for the json data to parse.
    - `log_file`: the file to log to.
    - `log_frequency`: Log information every `log_frequency` files.
4. Run the project.

## Progress

Progress is saved in `total.json` to avoid parsing the same files multiple times.
