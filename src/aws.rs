use conf::DropConfig;

use std::process::{Command, Stdio, exit};
use std::path::Path;

pub fn upload_file_to_s3(config: &DropConfig, file_path: &Path, file_name: Option<String>) {

  if !file_path.exists() {
    println!("File does not exist, nothing to upload to S3! ({:?})", file_path);
    exit(1);
  }
  
  let object_name = file_name.unwrap_or(file_path.file_name().unwrap().to_string_lossy().into_owned());
  let mut cmd = Command::new("s3cmd");
  cmd.arg("--force")
    .arg("--follow-symlinks")
    .arg(format!("--access_key={}", config.aws_key.clone().unwrap()))
    .arg(format!("--secret_key={}", config.aws_secret.clone().unwrap()))
    .arg("put")
    .arg(file_path.to_string_lossy().into_owned())
    .arg(format!("s3://{}/{}", config.aws_bucket.clone().unwrap(), object_name));

  if config.verbose {
    cmd.spawn().unwrap().wait();
  } else {
    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap().wait();
  }
}
