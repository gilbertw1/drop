extern crate config;
extern crate rand;
extern crate clap;
extern crate nix;
extern crate sys_info;
extern crate anyhow;
#[macro_use]
extern crate lazy_static;

#[cfg(target_os = "linux")]
extern crate gtk;
#[cfg(target_os = "linux")]
extern crate libappindicator;
#[cfg(target_os = "linux")]
extern crate libc;

extern { }

#[cfg(target_os = "macos")]
#[macro_use] extern crate objc;
#[cfg(target_os = "macos")]
extern crate cocoa;

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
extern { }
#[cfg(target_os = "macos")]
#[link(name = "Foundation", kind = "framework")]
extern { }
#[cfg(target_os = "macos")]
#[link(name = "AVFoundation", kind = "framework")]
extern { }
#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern { }

use std::path::{PathBuf, Path};
use clap::ArgMatches;
use std::io::{self, Read, Write};
use std::fs::File;

mod aws;
mod clip;
mod conf;
mod notify;
mod capture;
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
    handle_screen_capture(config, &matches);
  } else {
    let result = cli_app.print_help();
    if result.is_err() {
      println!("WARNING: Error occurred attempting to print help text")
    }
  }
}

fn handle_screen_capture(config: DropConfig, matches: &ArgMatches) {
  let out_file =
    if matches.is_present("video") {
      capture_screencast(&config)
    } else {
      capture_screenshot(&config)
    };

  let url = handle_upload_and_produce_url(&config, &out_file.as_path(), None);
  clip::copy_to_clipboard(url.clone());
  if config.notifications {
    notify::send_screenshot_notification(&out_file.as_path(), &config);
  }
  println!("{}", url);
}

fn capture_screenshot(config: &DropConfig) -> PathBuf {
  let out_file_name = util::generate_filename(config, None, Some("png".to_string()));
  let out_file = Path::new(&config.dir).join(out_file_name);
  capture::screenshot(out_file.as_path(), config);
  out_file
}

fn capture_screencast(config: &DropConfig) -> PathBuf {
  let out_file_name = util::generate_filename(config, None, Some(config.video_format.clone()));
  let out_file = Path::new(&config.dir).join(out_file_name);
  capture::screencast(out_file.as_path(), config);
  out_file
}

fn handle_file(config: DropConfig, matches: &ArgMatches) {
  let file = matches.value_of("file").unwrap();
  if file == "-" {
    handle_stdin(config);
  } else {
    handle_file_upload(config, Path::new(file));
  }
}

fn handle_file_upload(config: DropConfig, file: &Path) {
  if !file.exists() {
    println!("File does not exist! ({:?})", file);
    std::process::exit(1);
  } else {
    let filename = util::generate_filename(&config, file.file_name().map(|s| util::from_os_str(s)), None);
    let url = handle_upload_and_produce_url(&config, &file, Some(filename.clone()));
    clip::copy_to_clipboard(url.clone());
    if config.notifications {
      notify::send_upload_notification(filename, &config);
    }
    println!("{}", url);
  }
}

fn handle_stdin(config: DropConfig) {
  let mut buffer = Vec::new();

  let result = io::stdin().read_to_end(&mut buffer);
  if result.is_err() {
    println!("ERROR: Caught error while reading input from stdin");
    std::process::exit(1);
  }

  let out_filename = util::generate_filename(&config, None, None);
  let path = Path::new(&config.dir).join(out_filename.clone());
  let mut file = File::create(&path).unwrap();

  let write_result = file.write_all(&buffer);
  if write_result.is_err() {
    println!("ERROR: Caught error while writing to file");
    std::process::exit(1)
  }

  let url = handle_upload_and_produce_url(&config, &path, Some(out_filename.clone()));
  clip::copy_to_clipboard(url.clone());
  if config.notifications {
    notify::send_upload_notification(out_filename.clone(), &config);
  }
  println!("{}", url);
}

fn handle_upload_and_produce_url(config: &DropConfig, file: &Path, filename: Option<String>) -> String {
  if config.local || config.aws_bucket.is_none() || config.aws_key.is_none() || config.aws_secret.is_none() {
    format!("file://{}", util::path_to_str(file.canonicalize().unwrap().as_path()))
  } else {
    aws::upload_file_to_s3(&config, &file, &filename);
    util::create_drop_url(&config, filename.unwrap_or(util::from_os_str(file.file_name().unwrap())))
  }
}
