use std::process::{Command, Stdio};
use std::path::Path;

pub fn send_screenshot_notification(file_path: &Path) {
  Command::new("notify-send")
    .arg("-i")
    .arg(file_path.to_string_lossy().into_owned())
    .arg("'Drop url in clipboard.'")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn().unwrap().wait();
}

pub fn send_upload_notification(file_name: String) {
  Command::new("notify-send")
    .arg(format!("'Drop upload completed: {}'", file_name))
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn().unwrap().wait();
}
