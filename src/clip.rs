use std::process::{Command, Stdio};
use std::io::Write;

pub fn copy_to_clipboard(url: String) {
  let mut process = Command::new("xsel").arg("--clipboard").stdin(Stdio::piped()).spawn().unwrap();
  process.stdin.as_mut().unwrap().write_all(url.as_bytes());
  process.wait();
}
