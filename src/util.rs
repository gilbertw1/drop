use std::path::Path;
use rand;
use rand::Rng;
use std::ffi::OsStr;

use conf::DropConfig;

pub fn path_to_str(path: &Path) -> String {
  path.to_string_lossy().into_owned()
}

pub fn path_file_name(path: &Path) -> String {
  from_os_str(path.file_name().unwrap())
}

pub fn from_os_str(os_str: &OsStr) -> String {
  os_str.to_string_lossy().into_owned()
}

pub fn create_drop_url(config: &DropConfig, filename: String) -> String {
  match config.host.clone() {
    Some(host) => format!("http://{}/{}", host, filename),
    None => format!("http://s3.amazonaws.com/{}/{}", config.aws_bucket.clone().unwrap(), filename)
  }
}

pub fn generate_filename(config: &DropConfig, recommended_filename: Option<String>, recommended_ext: Option<String>) -> String {
  let file_base = generate_filename_base(config, recommended_filename.clone());
  let file_ext = generate_filename_extension(config, recommended_filename, recommended_ext);
  if file_ext.is_some() && !file_ext.clone().unwrap().is_empty() {
    format!("{}.{}", file_base, file_ext.unwrap())
  } else {
    file_base
  }
}

fn generate_filename_base(config: &DropConfig, recommended_filename: Option<String>) -> String {
  if config.filename.is_some() {
    create_filename_base_from_existing(config, config.filename.clone().unwrap())
  } else if recommended_filename.is_some() {
    create_filename_base_from_existing(config, recommended_filename.unwrap())
  } else {
    rand_string(config.unique_length)
  }
}

fn create_filename_base_from_existing(config: &DropConfig, filename: String) -> String {
  let file_base = filename.splitn(2, '.').next().unwrap();
  match config.filename_strategy.as_ref() {
    "exact" => file_base.to_string(),
    "append" => append_rand_string(file_base, config.unique_length),
    "replace" => rand_string(config.unique_length),
    _ => prepend_rand_string(file_base, config.unique_length),
  }
}

fn generate_filename_extension(config: &DropConfig, recommended_file_name: Option<String>, recommended_ext: Option<String>) -> Option<String> {
  if config.extension.is_some() {
    config.extension.clone()
  } else if config.filename.is_some() {
    config.filename.clone().unwrap().splitn(2, '.').nth(1).map(|s| s.to_string()).or(recommended_ext)
  } else if recommended_ext.is_some() {
    recommended_ext
  } else if recommended_file_name.is_some() {
    recommended_file_name.unwrap().splitn(2, '.').nth(1).or(Some("")).map(|s| s.to_string())
  } else {
    None
  }
}

fn prepend_rand_string(value: &str, len: usize) -> String {
  format!("{}--{}", rand_string(len), value)
}

fn append_rand_string(value: &str, len: usize) -> String {
  format!("{}--{}", value, rand_string(len))
}

fn rand_string(len: usize) -> String {
  rand::thread_rng().gen_ascii_chars().take(len).collect()
}
