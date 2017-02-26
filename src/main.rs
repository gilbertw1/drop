extern crate config;
extern crate rand;
extern crate clap;
extern crate gtk;
extern crate libc;

use std::path::{PathBuf, Path};
use clap::ArgMatches;

mod aws;
mod clip;
mod conf;
mod notify;
mod screenshot;
mod util;
mod cli;
mod ui;

use conf::DropConfig;

fn main() {
  let cli_app = cli::create_drop_cli_app();
  let matches = cli_app.get_matches();
  let config = conf::load_config(&matches);

  if matches.is_present("file") {
    handle_file_upload(config, matches);
  } else {
    handle_screenshot(config, matches)
  }
}

fn handle_screenshot(config: DropConfig, matches: ArgMatches) {
  let out_file =
    if matches.is_present("video") {
      take_screenshot_video(&config)
    } else {
      take_screenshot_image(&config)
    };

  if config.aws_bucket.is_none() || config.aws_key.is_none() || config.aws_secret.is_none() {
    println!("S3 not properly configured, not uploading screenshot.");
    clip::copy_to_clipboard(format!("file://{}", util::path_to_str(&out_file.as_path())));
    notify::send_screenshot_notification(&out_file.as_path());
  } else {
    aws::upload_file_to_s3(&config, &out_file.as_path(), None);
    let url = util::create_drop_url(&config, util::path_file_name(&out_file.as_path()));
    clip::copy_to_clipboard(url.clone());
    notify::send_screenshot_notification(&out_file.as_path());
    println!("{}", url);
  }
}

fn take_screenshot_image(config: &DropConfig) -> PathBuf {
  let out_file = util::gen_file(config.dir.clone(), "png", config.unique_length);
  screenshot::crop_and_take_screenshot(out_file.as_path());
  out_file
}

fn take_screenshot_video(config: &DropConfig) -> PathBuf {
  let out_file = util::gen_file(config.dir.clone(), &config.video_format, config.unique_length);
  screenshot::crop_and_take_screencast(out_file.as_path(), config.video_format.clone(), config.audio);
  out_file
}

fn handle_file_upload(config: DropConfig, matches: ArgMatches) {
  let file = Path::new(matches.value_of("file").unwrap());

  if !file.exists() {
    println!("File does not exist! ({:?})", file);
    std::process::exit(1);
  } else if config.aws_bucket.is_none() || config.aws_key.is_none() || config.aws_secret.is_none() {
    println!("S3 not properly configured, not uploading file.")
  } else {
    let filename = util::gen_filename_from_existing(file, config.filename_strategy.clone(), config.unique_length);
    aws::upload_file_to_s3(&config, &file, Some(filename.clone()));
    let url = util::create_drop_url(&config, filename.clone());
    clip::copy_to_clipboard(url.clone());
    notify::send_upload_notification(filename);
    println!("{}", url);
  }
}
