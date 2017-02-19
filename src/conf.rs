use std;
use config;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;

use util;

const DEFAULT_CONFIG: &'static str = include_str!("../config.toml.default");

pub fn load_config() -> DropConfig {
  let home_dir = std::env::home_dir().unwrap();
  let conf_file = home_dir.join(".config/drop/config.toml");

  if !conf_file.exists() {
    create_default_config_File(&conf_file)
  }

  let mut conf = config::Config::new();
  conf.merge(config::File::new(&util::path_to_str(&conf_file), config::FileFormat::Toml)).unwrap();

  let config = DropConfig {
    drop_dir: conf.get("drop.dir").unwrap().into_str().unwrap().replace("~", &home_dir.to_string_lossy().into_owned()),
    drop_host: conf.get("drop.host").map(|host| host.into_str().unwrap()),
    aws_bucket: conf.get("aws.bucket").map(|bucket| bucket.into_str().unwrap()),
    aws_key: conf.get("aws.key").map(|key| key.into_str().unwrap()),
    aws_secret: conf.get("aws.secret").map(|secret| secret.into_str().unwrap()),
  };

  ensure_directory_exists(&PathBuf::from(&config.drop_dir));
  config
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
  pub drop_dir: String,
  pub drop_host: Option<String>,
  pub aws_bucket: Option<String>,
  pub aws_key: Option<String>,
  pub aws_secret: Option<String>
}
