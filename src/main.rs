use crate::config::Config;
use chrono::{DateTime, Utc};
use clap::Parser;
use config::JobStart;
use std::sync::Arc;
use tokio::{
    process::Command,
    sync::Mutex,
    time::{self},
};

mod cli;
mod config;

struct SchedulerState {
    last_run_time: DateTime<Utc>,
}

impl SchedulerState {
    fn new(start_str: JobStart) -> Result<Self, chrono::ParseError> {
        let last_run_time =
            DateTime::parse_from_rfc3339(&start_str.to_string())?.with_timezone(&Utc);

        Ok(SchedulerState { last_run_time })
    }
}

async fn execute_job(state: Arc<Mutex<SchedulerState>>, config: &Config) {
    let mut state = state.lock().await;
    let start_time = state.last_run_time;
    let end_time = start_time + config.job.duration;

    println!("Mongo query timerange: {:?} - {:?}", start_time, end_time);

    // Update last run time for the next job
    state.last_run_time = end_time;

    let mongoexport_command = format!(
        "mongoexport --uri='mongodb://mongoadmin:secret@localhost:27017/mydatabase?authSource=admin' \
         --collection='products' --query='{{\"updatedAt\": {{\"$gte\": {{\"$date\": \"{}\"}}, \
         \"$lte\": {{\"$date\": \"{}\"}}}}}}' --out='products.out.json'",
        start_time.to_rfc3339(),
        end_time.to_rfc3339(),
    );

    // Execute the mongoexport command
    let mongo_output = Command::new("sh")
        .arg("-c")
        .arg(mongoexport_command.clone())
        .output()
        .await
        .expect(&format!(
            "Failed to execute command: {:?}",
            mongoexport_command
        ));

    println!(
        "mongoexport output: {:?}",
        String::from_utf8_lossy(&mongo_output.stderr)
    );

    // Import to Postgres
    let psql_import_command = format!(
        "psql {} -c \"\\copy public.products_raw FROM 'products.out.json' WITH (FORMAT csv, QUOTE E'\\x01', DELIMITER E'\\x02')\"",
        config.destination.postgres_uri
    );

    let psql_output = Command::new("sh")
        .arg("-c")
        .arg(&psql_import_command)
        .output()
        .await
        .expect("Failed to execute PostgreSQL import command");

    if !psql_output.status.success() {
        let error_message = String::from_utf8_lossy(&psql_output.stderr);
        println!("PostgreSQL import failed: {}", error_message);
    } else {
        println!("PostgreSQL import executed successfully");
    }

    println!("------------------");
}

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    let config = Config::new(&args.config).expect("Failed to load config");

    println!("Running with config:\n {:?}", config);
    let duration = config.job.duration;
    println!("Schedule: {}", duration);

    let state = Arc::new(Mutex::new(
        SchedulerState::new(config.job.start.clone()).unwrap(),
    ));

    let interval_duration = time::Duration::from_secs(duration.num_seconds() as u64);

    // Initial run
    println!("------------------");
    execute_job(state.clone(), &config).await;

    // Subsequent runs
    let mut interval = time::interval(interval_duration);
    loop {
        interval.tick().await;
        execute_job(state.clone(), &config).await;
    }
}
