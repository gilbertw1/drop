use util;

use std;
use config;
use config::Config;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use clap::ArgMatches;

const DEFAULT_CONFIG: &'static str = include_str!("../config.toml.default");

pub fn load_config(matches: &ArgMatches) -> DropConfig {
  let home_dir = std::env::home_dir().unwrap();
  let conf_file = home_dir.join(".config/drop/config.toml");

  if !conf_file.exists() {
    create_default_config_file(&conf_file)
  }

  let mut conf = Config::new();
  conf.merge(config::File::new(&util::path_to_str(&conf_file), config::FileFormat::Toml)).unwrap();

  let config = DropConfig {
    dir: conf.get_str("drop.dir").ok().unwrap_or("~/.drop".to_string()).replace("~", &home_dir.to_string_lossy().into_owned()),
    host: none_if_empty(get_string_value(matches, "host").or(conf.get_str("drop.host").ok())),
    aws_bucket: get_string_value(matches, "aws-bucket").or(conf.get_str("aws.bucket").ok()),
    aws_key: get_string_value(matches, "aws-key").or(conf.get_str("aws.key").ok()),
    aws_secret: get_string_value(matches, "aws-secret").or(conf.get_str("aws.secret").ok()),
    filename_strategy: extract_strategy(get_string_value(matches, "filename-strategy").or(conf.get_str("drop.filename_strategy").ok())),
    unique_length: get_string_value(matches, "unique-length").map(|ls| ls.parse::<usize>().unwrap())
      .or(conf.get_int("drop.unique_length").ok().map(|i| i as usize)) .unwrap_or(10),
    transparent: get_bool_value(matches, "transparent", conf.get_bool("drop.transparent").unwrap_or(false)),
    tray_icon: get_bool_value(matches, "tray-icon", conf.get_bool("drop.tray_icon").unwrap_or(true)),
    stop_key: get_string_value(matches, "stop-key").or(conf.get_str("drop.stop_key").ok()),
    notifications: get_bool_value(matches, "notifications", conf.get_bool("drop.notifications").unwrap_or(true)),
    filename: get_string_value(matches, "filename"),
    extension: get_string_value(matches, "extension"),
    audio: get_bool_value(matches, "audio", false),
    border: get_bool_value(matches, "border", true),
    mouse: get_bool_value(matches, "mouse", false),
    video_format: get_video_format(matches),
    verbose: matches.is_present("verbose"),
  };

  ensure_directory_exists(&PathBuf::from(&config.dir));
  config
}

fn extract_strategy(strat: Option<String>) -> String {
  if strat.is_none() {
    "prepend".to_string()
  } else {
    let strat = strat.unwrap();
    if strat.to_lowercase() == "exact" || strat.to_lowercase() == "append" ||
      strat.to_lowercase() == "replace" || strat.to_lowercase() == "prepend" {
        strat.to_lowercase()
    } else {
      panic!(format!("Unrecognized filename strategy: {}", strat))
    }
  }
}

fn none_if_empty(optvalue: Option<String>) -> Option<String> {
  match optvalue {
    Some(ref value) if value != "" => Some(value.to_string()),
    _ => None,
  }
}

fn get_string_value(matches: &ArgMatches, key: &str) -> Option<String> {
  matches.value_of(key).map(|m| m.to_string())
}

fn get_bool_value(matches: &ArgMatches, key: &str, default: bool) -> bool {
  matches.value_of(key).unwrap_or(&format!("{}", default)).parse::<bool>().unwrap_or(default)
}

fn get_video_format(matches: &ArgMatches) -> String {
  match matches.value_of("video-format") {
    Some("gif") => "gif".to_string(),
    _ => "mp4".to_string(),
  }
}

fn ensure_directory_exists(dir: &PathBuf) {
  let result = std::fs::create_dir_all(dir);
  if result.is_err() {
    println!("ERROR: Failed to ensure drop directory exists");
    std::process::exit(1);
  }
}

fn create_default_config_file(config_file_path: &PathBuf) {
  ensure_directory_exists(&config_file_path.parent().unwrap().to_path_buf());
  let mut file = File::create(config_file_path).unwrap();
  let result = file.write_all(DEFAULT_CONFIG.as_bytes());
  if result.is_err() {
    println!("WARNING: Failed to create default config file");
  }
}

#[derive(Debug)]
pub struct DropConfig {
  pub dir: String,
  pub host: Option<String>,
  pub aws_bucket: Option<String>,
  pub aws_key: Option<String>,
  pub aws_secret: Option<String>,
  pub filename_strategy: String,
  pub unique_length: usize,
  pub transparent: bool,
  pub tray_icon: bool,
  pub stop_key: Option<String>,
  pub notifications: bool,

  // CLI Only Options
  pub audio: bool,
  pub border: bool,
  pub extension: Option<String>,
  pub filename: Option<String>,
  pub mouse: bool,
  pub video_format: String,
  pub verbose: bool,
}
