extern crate config;
extern crate rand;
extern crate clap;

use std::path::PathBuf;
use rand::Rng;

mod aws;
mod clip;
mod conf;
mod notify;
mod screenshot;

use conf::DropConfig;

fn main() {
  let config = conf::load_config();
  std::fs::create_dir_all(&config.drop_dir);
  let out_filename = gen_filename("png");
  let mut out_file_path = PathBuf::from(&config.drop_dir);
  out_file_path.push(&out_filename);

  let success = screenshot::crop_and_take_screenshot(&out_file_path);

  if !success {
    std::process::exit(1);
  }

  if config.aws_bucket.is_none() || config.aws_key.is_none() || config.aws_secret.is_none() {
    println!("AWS not properly defined, not uploading.");
    clip::copy_to_clipboard(format!("file://{}", out_file_path.to_string_lossy().into_owned()));
    notify::send_notification(&out_file_path);
  } else {
    aws::upload_file_to_s3(&config, &out_file_path);
    let url = create_drop_url(&config, out_filename);
    clip::copy_to_clipboard(url);
    notify::send_notification(&out_file_path);
  }
}

fn gen_filename(ext: &str) -> String {
  let rand_filename: String = rand::thread_rng().gen_ascii_chars().take(10).collect();
  format!("{}.{}", rand_filename, ext)
}

fn create_drop_url(config: &DropConfig, filename: String) -> String {
  match config.drop_host.clone() {
    Some(host) => format!("http://{}/{}", host, filename),
    None => format!("http://s3.amazonaws.com/{}/{}", config.aws_bucket.clone().unwrap(), filename)
  }
}
