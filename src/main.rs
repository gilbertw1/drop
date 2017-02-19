extern crate config;
extern crate rand;
extern crate clap;

use std::path::{PathBuf, Path};
use clap::ArgMatches;

mod aws;
mod clip;
mod conf;
mod notify;
mod screenshot;
mod util;
mod cli;

use conf::DropConfig;

fn main() {
  let cli_app = cli::create_drop_cli_app();
  let matches = cli_app.get_matches();
  let config = conf::load_config();

  if matches.is_present("file") {
    handle_file_upload(config, matches);
  } else {
    handle_screenshot(config, matches)
  }
}

fn handle_screenshot(config: DropConfig, matches: ArgMatches) {
  let out_file = util::gen_file(config.drop_dir.clone(), "png");

  let success = screenshot::crop_and_take_screenshot(out_file.as_path());

  if !success {
    std::process::exit(1);
  }

  if config.aws_bucket.is_none() || config.aws_key.is_none() || config.aws_secret.is_none() {
    println!("AWS not properly defined, not uploading.");
    clip::copy_to_clipboard(format!("file://{}", util::path_to_str(&out_file.as_path())));
    notify::send_screenshot_notification(&out_file.as_path());
  } else {
    aws::upload_file_to_s3(&config, &out_file.as_path(), None);
    let url = util::create_drop_url(&config, util::path_file_name(&out_file.as_path()));
    clip::copy_to_clipboard(url);
    notify::send_screenshot_notification(&out_file.as_path());
  }
}

fn handle_file_upload(config: DropConfig, matches: ArgMatches) {
  let file = Path::new(matches.value_of("file").unwrap());

  if !file.exists() {
    println!("File does not exist! ({:?})", file);
  } else {
    let filename = util::gen_filename_from_existing(file, 10);
    aws::upload_file_to_s3(&config, &file, Some(filename.clone()));
    let url = util::create_drop_url(&config, filename.clone());
    clip::copy_to_clipboard(url);
    notify::send_upload_notification(filename);
  }
}
