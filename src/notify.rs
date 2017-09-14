use conf::DropConfig;
use util;

use std::process::Command;
use std::path::Path;

#[cfg(target_os = "linux")]
pub fn send_screenshot_notification(file_path: &Path, config: &DropConfig) {
  let mut cmd = Command::new("notify-send");
  cmd.arg("-i")
    .arg(file_path.to_string_lossy().into_owned())
    .arg("Drop complete")
    .arg("Url is in clipboard.");

  let result = util::run_command_and_wait(&mut cmd, "NOTIFY SEND", config);
  if !result.success() {
    println!("WARNING: Failed to create desktop notification");
  }
}

#[cfg(target_os = "linux")]
pub fn send_upload_notification(file_name: String, config: &DropConfig) {
  let mut cmd = Command::new("notify-send");
  cmd.arg(format!("Drop complete"))
    .arg(format!("Url is in clipboard ({})", file_name));

  let result = util::run_command_and_wait(&mut cmd, "NOTIFY SEND", config);
  if !result.success() {
    println!("WARNING: Failed to create desktop notification");
  }
}


#[cfg(target_os = "macos")]
pub fn send_screenshot_notification(file_path: &Path, config: &DropConfig) {
  let mut cmd = Command::new("osascript");
  cmd.arg("-e").arg(format!("display notification \"Drop url in clipboard.\" with title \"drop\""));

  let result = util::run_command_and_wait(&mut cmd, "OSA SCRIPT", config);
  if !result.success() {
    println!("WARNING: Failed to create desktop notification");
  }
}


#[cfg(target_os = "macos")]
pub fn send_upload_notification(file_name: String, config: &DropConfig) {
  let mut cmd = Command::new("osascript");
  cmd.arg("-e").arg(format!("display notification \"Drop upload completed: {}\" with title \"drop\"", file_name));

  let result = util::run_command_and_wait(&mut cmd, "OSA SCRIPT", config);
  if !result.success() {
    println!("WARNING: Failed to create desktop notification");
  }
}
