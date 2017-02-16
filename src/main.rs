extern crate config;
extern crate rand;

use std::env;
use std::string::String;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::io::Write;
use rand::Rng;

fn main() {
  let config = load_config();
  std::fs::create_dir_all(&config.drop_dir);
  let out_filename = gen_filename("png");
  let mut out_file_path = PathBuf::from(&config.drop_dir);
  out_file_path.push(&out_filename);
  let slop_result = run_slop();

  if slop_result.cancel {
    println!("Cancelled drop, exiting.");
    std::process::exit(1);
  }

  take_and_crop_screenshot(&slop_result, &out_file_path);

  if config.aws_bucket.is_none() || config.aws_key.is_none() || config.aws_secret.is_none() {
    println!("AWS not properly defined, not uploading.");
    copy_to_clipboard(format!("file://{}", out_file_path.to_string_lossy().into_owned()));
    send_notification(&out_file_path);
  }

  upload_file_to_s3(&config, &out_file_path);

  let url = create_drop_url(&config, out_filename);
  copy_to_clipboard(url);
  send_notification(&out_file_path);
}

fn gen_filename(ext: &str) -> String {
  let rand_filename: String = rand::thread_rng().gen_ascii_chars().take(10).collect();
  format!("{}.{}", rand_filename, ext)
}

fn load_config() -> DropConfig {
  let home_dir = env::home_dir().unwrap();
  let mut conf = config::Config::new();
  let conf_file = home_dir.join(".config/drop/drop.conf").to_string_lossy().into_owned();
  conf.merge(config::File::new(conf_file.as_str(), config::FileFormat::Toml)).unwrap();
  DropConfig {
    drop_dir: conf.get("drop.dir").unwrap().into_str().unwrap().replace("~", &home_dir.to_string_lossy().into_owned()),
    drop_host: conf.get("drop.host").map(|host| host.into_str().unwrap()),
    aws_bucket: conf.get("aws.bucket").map(|bucket| bucket.into_str().unwrap()),
    aws_key: conf.get("aws.key").map(|key| key.into_str().unwrap()),
    aws_secret: conf.get("aws.secret").map(|secret| secret.into_str().unwrap()),
  }
}

fn run_slop() -> SlopResult {
  let result = Command::new("slop").args(&["--color", "0.275,0.510,0.706"]).output().unwrap();
  let output = String::from_utf8(result.stdout).unwrap();
  let out_map = output.trim()
                      .split("\n")
                      .map(|kv| kv.split("="))
                      .map(|mut kv| (kv.next().unwrap().into(), kv.next().unwrap().into()))
                      .collect::<HashMap<String, String>>();

    SlopResult {
      x: out_map.get("X").unwrap().clone(),
      y: out_map.get("Y").unwrap().clone(),
      w: out_map.get("W").unwrap().clone(),
      h: out_map.get("H").unwrap().clone(),
      g: out_map.get("G").unwrap().clone(),
      id: out_map.get("ID").unwrap().clone(),
      cancel: out_map.get("Cancel").unwrap().clone() == "true",
    }
}

fn take_and_crop_screenshot(slop_result: &SlopResult, out_path: &PathBuf) {
  Command::new("import").args(&["-window", "root", "-crop", &slop_result.g, &out_path.to_string_lossy().into_owned()]).spawn().unwrap().wait();
}

fn upload_file_to_s3(config: &DropConfig, file_path: &PathBuf) {
  Command::new("s3cmd")
    .arg("--force")
    .arg(format!("--access_key={}", config.aws_key.clone().unwrap()))
    .arg(format!("--secret_key={}", config.aws_secret.clone().unwrap()))
    .arg("put")
    .arg(file_path.to_string_lossy().into_owned())
    .arg(format!("s3://{}", config.aws_bucket.clone().unwrap()))
    .output();
}

fn create_drop_url(config: &DropConfig, filename: String) -> String {
  match config.drop_host.clone() {
    Some(host) => format!("http://{}/{}", host, filename),
    None => format!("http://s3.amazonaws.com/{}/{}", config.aws_bucket.clone().unwrap(), filename)
  }
}

fn copy_to_clipboard(url: String) {
  let mut process = Command::new("xsel").arg("--clipboard").stdin(Stdio::piped()).spawn().unwrap();
  process.stdin.as_mut().unwrap().write_all(url.as_bytes());
  process.wait();
}

fn send_notification(file_path: &PathBuf) {
  Command::new("notify-send").arg("-i").arg(file_path.to_string_lossy().into_owned()).arg("'Drop url in clipboard.'").spawn().unwrap().wait();
}

#[derive(Debug)]
struct SlopResult {
    x: String,
    y: String,
    w: String,
    h: String,
    g: String,
    id: String,
    cancel: bool,
}

#[derive(Debug)]
struct DropConfig {
  drop_dir: String,
  drop_host: Option<String>,
  aws_bucket: Option<String>,
  aws_key: Option<String>,
  aws_secret: Option<String>
}
