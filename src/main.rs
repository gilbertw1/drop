extern crate config;
extern crate rand;
extern crate clap;
extern crate libc;
#[cfg(target_os = "linux")]
extern crate gtk;

use std::path::{PathBuf, Path};
use clap::ArgMatches;
use std::io::{self, Read, Write};
use std::fs::File;

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
  
  let mut cli_app = cli::create_drop_cli_app();
  let matches = cli_app.clone().get_matches();
  let config = conf::load_config(&matches);
  
  if matches.is_present("file") {
    handle_file(config, &matches);
  } else if matches.is_present("screenshot") || matches.is_present("video") {
    handle_screenshot(config, &matches);
  } else {
    cli_app.print_help();
  }
}

fn handle_screenshot(config: DropConfig, matches: &ArgMatches) {
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
  let out_file_name = util::generate_filename(config, None, Some("png".to_string()));
  let out_file = Path::new(&config.dir).join(out_file_name);
  screenshot::crop_and_take_screenshot(out_file.as_path(), config.transparent);
  out_file
}

fn take_screenshot_video(config: &DropConfig) -> PathBuf {
  let out_file_name = util::generate_filename(config, None, Some(config.video_format.clone()));
  let out_file = Path::new(&config.dir).join(out_file_name);
  screenshot::crop_and_take_screencast(out_file.as_path(), config.video_format.clone(), config.audio, config.transparent);
  out_file
}

fn handle_file(config: DropConfig, matches: &ArgMatches) {
  let file = matches.value_of("file").unwrap();
  if file == "-" {
    handle_stdin(config, matches);
  } else {
    handle_file_upload(config, matches, Path::new(file));
  }
}

fn handle_file_upload(config: DropConfig, matches: &ArgMatches, file: &Path) {
  if !file.exists() {
    println!("File does not exist! ({:?})", file);
    std::process::exit(1);
  } else if config.aws_bucket.is_none() || config.aws_key.is_none() || config.aws_secret.is_none() {
    println!("S3 not properly configured, not uploading file.")
  } else {
    let filename = util::generate_filename(&config, file.file_name().map(|s| util::from_os_str(s)), None);
    aws::upload_file_to_s3(&config, &file, Some(filename.clone()));
    let url = util::create_drop_url(&config, filename.clone());
    clip::copy_to_clipboard(url.clone());
    notify::send_upload_notification(filename);
    println!("{}", url);
  }
}

fn handle_stdin(config: DropConfig, matches: &ArgMatches) {
  let reader = io::stdin();
  let mut buffer = Vec::new();
  io::stdin().read_to_end(&mut buffer);
  let out_filename = util::generate_filename(&config, None, None);
  let path = Path::new(&config.dir).join(out_filename.clone());
  let mut file = File::create(&path).unwrap();
  file.write_all(&buffer);
  aws::upload_file_to_s3(&config, &path, Some(out_filename.clone()));
  let url = util::create_drop_url(&config, out_filename.clone());
  clip::copy_to_clipboard(url.clone());
  notify::send_upload_notification(out_filename.clone());
  println!("{}", url);
}
