use chrono::Duration as ChronoDuration;
use chrono::Utc;
use duration_string::DurationString;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::fmt;
use std::time::Duration as StdDuration;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub mongo_uri: String,
    pub mongo_collection: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Destination {
    pub postgres_uri: String,
    pub postgres_table: String,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub name: String,
    pub start: JobStart,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub duration: ChronoDuration,
}

#[derive(Debug, Serialize)]
pub struct JobStart(String);
impl Default for JobStart {
    fn default() -> Self {
        JobStart(Utc::now().to_rfc3339())
    }
}

impl From<String> for JobStart {
    fn from(s: String) -> Self {
        JobStart(s)
    }
}

// You can also implement From<&str> for convenience
impl From<&str> for JobStart {
    fn from(s: &str) -> Self {
        JobStart(s.to_owned())
    }
}

impl fmt::Display for JobStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobIntermediate {
    pub name: String,
    pub start: Option<String>,
    pub schedule: i64,
    pub schedule_unit: String,
}

fn convert_std_duration_to_chrono(std_duration: StdDuration) -> ChronoDuration {
    let seconds = std_duration.as_secs() as i64;
    let nanoseconds = std_duration.subsec_nanos() as i32;

    ChronoDuration::seconds(seconds) + ChronoDuration::nanoseconds(i64::from(nanoseconds))
}

impl<'de> Deserialize<'de> for Job {
    fn deserialize<D>(deserializer: D) -> Result<Job, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inter = JobIntermediate::deserialize(deserializer)?;

        let duration: StdDuration = format!("{}{}", inter.schedule, inter.schedule_unit)
            .parse::<DurationString>()
            .expect(&format!(
                "Failed to parse duration from value {}{} ",
                inter.schedule, inter.schedule_unit
            ))
            .into();
        let duration = convert_std_duration_to_chrono(duration);
        // let start: JobStart = inter.start.into();

        let start: JobStart = match inter.start {
            Some(s) => s.into(),
            None => JobStart::default(),
        };

        Ok(Job {
            name: inter.name,
            start,
            duration,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub job: Job,
    pub source: Source,
    pub destination: Destination,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_file = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_file)?;
        Ok(config)
    }
}
