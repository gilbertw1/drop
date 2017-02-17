extern crate config;
extern crate rand;
extern crate clap;

use std::path::PathBuf;

mod aws;
mod clip;
mod conf;
mod notify;
mod screenshot;
mod util;

use conf::DropConfig;

fn main() {
  let config = conf::load_config();
  let out_file = util::gen_file(config.drop_dir.clone(), "png");

  let success = screenshot::crop_and_take_screenshot(&out_file);

  if !success {
    std::process::exit(1);
  }

  if config.aws_bucket.is_none() || config.aws_key.is_none() || config.aws_secret.is_none() {
    println!("AWS not properly defined, not uploading.");
    clip::copy_to_clipboard(format!("file://{}", util::path_to_str(&out_file)));
    notify::send_notification(&out_file);
  } else {
    aws::upload_file_to_s3(&config, &out_file);
    let url = util::create_drop_url(&config, util::path_file_name(&out_file));
    clip::copy_to_clipboard(url);
    notify::send_notification(&out_file);
  }
}
