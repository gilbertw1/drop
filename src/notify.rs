use std::process::Command;
use std::path::PathBuf;

pub fn send_notification(file_path: &PathBuf) {
  Command::new("notify-send")
    .arg("-i")
    .arg(file_path.to_string_lossy().into_owned())
    .arg("'Drop url in clipboard.'")
    .spawn().unwrap().wait();
}
