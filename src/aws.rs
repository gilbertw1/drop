use std::process::Command;
use std::path::PathBuf;

use conf::DropConfig;

pub fn upload_file_to_s3(config: &DropConfig, file_path: &PathBuf) {
  Command::new("s3cmd")
    .arg("--force")
    .arg(format!("--access_key={}", config.aws_key.clone().unwrap()))
    .arg(format!("--secret_key={}", config.aws_secret.clone().unwrap()))
    .arg("put")
    .arg(file_path.to_string_lossy().into_owned())
    .arg(format!("s3://{}", config.aws_bucket.clone().unwrap()))
    .output();
}
