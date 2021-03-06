use conf::DropConfig;

use std::path::Path;
use std::ffi::OsStr;
use std::{thread, time};
use std::io::{BufReader, BufRead};
use std::process::{Command, Child, Stdio, ExitStatus};
use rand;
use rand::Rng;
use rand::distributions::Alphanumeric;

pub fn path_to_str(path: &Path) -> String {
  path.to_string_lossy().into_owned()
}

pub fn from_os_str(os_str: &OsStr) -> String {
  os_str.to_string_lossy().into_owned()
}

pub fn wait_delay(config: &DropConfig) {
  if config.delay > 0 {
    thread::sleep(time::Duration::from_secs(config.delay));
  }
}

pub fn create_drop_url(config: &DropConfig, filename: String) -> String {
  match config.host.clone() {
    Some(host) => format!("http://{}/{}", host, filename),
    None => format!("http://s3.amazonaws.com/{}/{}", config.aws_bucket.clone().unwrap(), filename)
  }
}

pub fn generate_filename(config: &DropConfig, recommended_filename: Option<String>, recommended_ext: Option<String>) -> String {
  let file_base = generate_filename_base(config, recommended_filename.clone());
  let file_ext = generate_filename_extension(config, recommended_filename, recommended_ext);
  if file_ext.is_some() && !file_ext.clone().unwrap().is_empty() {
    format!("{}.{}", file_base, file_ext.unwrap())
  } else {
    file_base
  }
}

fn generate_filename_base(config: &DropConfig, recommended_filename: Option<String>) -> String {
  if config.filename.is_some() {
    create_filename_base_from_existing(config, config.filename.clone().unwrap())
  } else if recommended_filename.is_some() {
    create_filename_base_from_existing(config, recommended_filename.unwrap())
  } else {
    rand_string(config.unique_length)
  }
}

fn create_filename_base_from_existing(config: &DropConfig, filename: String) -> String {
  let file_base = filename.splitn(2, '.').next().unwrap();
  match config.filename_strategy.as_ref() {
    "exact" => file_base.to_string(),
    "append" => append_rand_string(file_base, config.unique_length),
    "replace" => rand_string(config.unique_length),
    _ => prepend_rand_string(file_base, config.unique_length),
  }
}

fn generate_filename_extension(config: &DropConfig, recommended_file_name: Option<String>, recommended_ext: Option<String>) -> Option<String> {
  if config.extension.is_some() {
    config.extension.clone()
  } else if config.filename.is_some() {
    config.filename.clone().unwrap().splitn(2, '.').nth(1).map(|s| s.to_string()).or(recommended_ext)
  } else if recommended_ext.is_some() {
    recommended_ext
  } else if recommended_file_name.is_some() {
    recommended_file_name.unwrap().splitn(2, '.').nth(1).or(Some("")).map(|s| s.to_string())
  } else {
    None
  }
}

fn prepend_rand_string(value: &str, len: usize) -> String {
  format!("{}--{}", rand_string(len), value)
}

fn append_rand_string(value: &str, len: usize) -> String {
  format!("{}--{}", value, rand_string(len))
}

pub fn rand_string(len: usize) -> String {
  rand::thread_rng().sample_iter(Alphanumeric).take(len).map(char::from).collect()
}

pub fn run_command_and_wait(cmd: &mut Command, name: &str, config: &DropConfig) -> ExitStatus {
  run_command(cmd, name, config).wait().unwrap()
}

pub fn run_command(cmd: &mut Command, name: &str, config: &DropConfig) -> Child {
  if config.verbose {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    log_child_output_to_stdout(&mut child, name);
    child
  } else {
    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap()
  }
}

fn log_child_output_to_stdout(child: &mut Child, name: &str) {
  let name_out = name.to_owned();
  let name_err = name.to_owned();
  let mut stdout = child.stdout.take();
  let mut stderr = child.stderr.take();
  
  thread::spawn(move || {
    if let Some(ref mut stdout) = stdout {
      for line in BufReader::new(stdout).lines() {
        let line = line.unwrap();
        println!("[{}] {}", name_out, line);
      }
    }
  });

  thread::spawn(move || {
    if let Some(ref mut stderr) = stderr {
      for line in BufReader::new(stderr).lines() {
        let line = line.unwrap();
        println!("[{}] {}", name_err, line);
      }
    }
  });
}
