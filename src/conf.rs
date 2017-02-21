use std;
use config;
use config::Config;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use clap::ArgMatches;

use util;

const DEFAULT_CONFIG: &'static str = include_str!("../config.toml.default");

pub fn load_config(matches: &ArgMatches) -> DropConfig {
  let home_dir = std::env::home_dir().unwrap();
  let conf_file = home_dir.join(".config/drop/config.toml");

  if !conf_file.exists() {
    create_default_config_File(&conf_file)
  }

  let mut conf = Config::new();
  conf.merge(config::File::new(&util::path_to_str(&conf_file), config::FileFormat::Toml)).unwrap();

  let config = DropConfig {
    dir: conf.get_str("drop.dir").unwrap_or("~/.drop".to_string()).replace("~", &home_dir.to_string_lossy().into_owned()),
    host: get_string_value(matches, "host").or(conf.get_str("drop.host")),
    aws_bucket: get_string_value(matches, "aws-bucket").or(conf.get_str("aws.bucket")),
    aws_key: get_string_value(matches, "aws-key").or(conf.get_str("aws.key")),
    aws_secret: get_string_value(matches, "aws-secret").or(conf.get_str("aws.secret")),
    filename_strategy: extract_strategy(get_string_value(matches, "filename-strategy").or(conf.get_str("drop.filename_strategy"))),
    unique_length: get_string_value(matches, "unique-length").map(|ls| ls.parse::<usize>().unwrap())
                     .or(conf.get_int("drop.unique_length").map(|i| i as usize)) .unwrap_or(10),
  };

  ensure_directory_exists(&PathBuf::from(&config.dir));
  config
}

fn extract_strategy(strat: Option<String>) -> String {
  if strat.is_none() {
    "APPEND".to_string()
  } else {
    let strat = strat.unwrap();
    if strat == "EXACT" || strat == "APPEND" || strat == "REPLACE" {
      strat
    } else {
      panic!(format!("Unrecognized filename strategy: {}", strat))
    }
  }
}

fn get_string_value(matches: &ArgMatches, key: &str) -> Option<String> {
  matches.value_of(key).map(|m| m.to_string())
}

fn ensure_directory_exists(dir: &PathBuf) {
  std::fs::create_dir_all(dir);
}

fn create_default_config_File(config_file_path: &PathBuf) {
  ensure_directory_exists(&config_file_path.parent().unwrap().to_path_buf());
  let mut f = File::create(config_file_path).unwrap();
  f.write_all(DEFAULT_CONFIG.as_bytes());
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
}
