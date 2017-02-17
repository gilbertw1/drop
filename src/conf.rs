use std;
use config;

pub fn load_config() -> DropConfig {
  let home_dir = std::env::home_dir().unwrap();
  let mut conf = config::Config::new();
  let conf_file = home_dir.join(".config/drop/config.toml").to_string_lossy().into_owned();
  conf.merge(config::File::new(conf_file.as_str(), config::FileFormat::Toml)).unwrap();
  let config = DropConfig {
    drop_dir: conf.get("drop.dir").unwrap().into_str().unwrap().replace("~", &home_dir.to_string_lossy().into_owned()),
    drop_host: conf.get("drop.host").map(|host| host.into_str().unwrap()),
    aws_bucket: conf.get("aws.bucket").map(|bucket| bucket.into_str().unwrap()),
    aws_key: conf.get("aws.key").map(|key| key.into_str().unwrap()),
    aws_secret: conf.get("aws.secret").map(|secret| secret.into_str().unwrap()),
  };

  ensure_directory_exists(&config.drop_dir);
  config
}

fn ensure_directory_exists(dir: &String) {
  std::fs::create_dir_all(dir);
}

#[derive(Debug)]
pub struct DropConfig {
  pub drop_dir: String,
  pub drop_host: Option<String>,
  pub aws_bucket: Option<String>,
  pub aws_key: Option<String>,
  pub aws_secret: Option<String>
}
