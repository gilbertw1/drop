use std::process::{Command, Stdio};
use std::io::Write;

#[cfg(target_os = "linux")]
pub fn copy_to_clipboard(url: String) {
  let mut process = Command::new("xsel").arg("--clipboard").stdin(Stdio::piped()).spawn().unwrap();
  let write_result = process.stdin.as_mut().unwrap().write_all(url.as_bytes());
  let result = process.wait();
  
  if result.is_err() || write_result.is_err() || !result.unwrap().success() {
    println!("WARNING: Failed to copy url to clipboard: {}", url);
  }
}

#[cfg(target_os = "macos")]
pub fn copy_to_clipboard(url: String) {
  let mut process = Command::new("pbcopy").stdin(Stdio::piped()).spawn().unwrap();
  let write_result = process.stdin.as_mut().unwrap().write_all(url.as_bytes());
  let result = process.wait();

  if result.is_err() || write_result.is_err() || !result.unwrap().success() {
    println!("WARNING: Failed to copy url to clipboard: {}", url);
  }
}
