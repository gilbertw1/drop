use std::process::Command;
use std::path::Path;

use conf::DropConfig;

pub fn upload_file_to_s3(config: &DropConfig, file_path: &Path, file_name: Option<String>) {
  let mut cmd = Command::new("s3cmd");
  cmd.arg("--force")
    .arg(format!("--access_key={}", config.aws_key.clone().unwrap()))
    .arg(format!("--secret_key={}", config.aws_secret.clone().unwrap()))
    .arg("put")
    .arg(file_path.to_string_lossy().into_owned());

  if file_name.is_some() {
    cmd.arg(format!("s3://{}/{}", config.aws_bucket.clone().unwrap(), file_name.unwrap()));
  } else {
    cmd.arg(format!("s3://{}", config.aws_bucket.clone().unwrap()));
  }

  cmd.spawn().unwrap().wait();
}
