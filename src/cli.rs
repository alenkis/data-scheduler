pub(crate) use clap::Parser;

/// Simple program to export data from MongoDB and import it into PostgreSQL.
/// Allows for scheduling of jobs.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path of the configuration file
    #[arg(short, long, default_value = "config.yml")]
    pub config: String,
}
