use std::process::{Command, Stdio};
use std::path::Path;

#[cfg(target_os = "linux")]
pub fn send_screenshot_notification(file_path: &Path) {
  Command::new("notify-send")
    .arg("-i")
    .arg(file_path.to_string_lossy().into_owned())
    .arg("Drop complete")
    .arg("Url is in clipboard.")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn().unwrap().wait();
}

#[cfg(target_os = "linux")]
pub fn send_upload_notification(file_name: String) {
  Command::new("notify-send")
    .arg(format!("Drop complete"))
    .arg(format!("Url is in clipboard ({})", file_name))
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn().unwrap().wait();
}


#[cfg(target_os = "macos")]
pub fn send_screenshot_notification(file_path: &Path) {
  Command::new("osascript")
    .arg("-e")
    .arg(format!("display notification \"Drop url in clipboard.\" with title \"drop\""))
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn().unwrap().wait();
}


#[cfg(target_os = "macos")]
pub fn send_upload_notification(file_name: String) {
  Command::new("osascript")
    .arg("-e")
    .arg(format!("display notification \"Drop upload completed: {}\" with title \"drop\"", file_name))
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn().unwrap().wait();
}
