use std::path::{Path, PathBuf};
use rand;
use rand::Rng;
use std::ffi::OsStr;

use conf::DropConfig;

pub fn path_to_str(path: &Path) -> String {
  path.to_string_lossy().into_owned()
}

pub fn path_file_name(path: &Path) -> String {
  from_os_str(path.file_name().unwrap())
}

fn path_file_ext(path: &Path) -> Option<String> {
  path.extension().map(|ext| from_os_str(ext))
}

fn path_file_stem(path: &Path) -> String {
  from_os_str(path.file_stem().unwrap())
}

fn from_os_str(os_str: &OsStr) -> String {
  os_str.to_string_lossy().into_owned()
}

pub fn create_drop_url(config: &DropConfig, filename: String) -> String {
  match config.host.clone() {
    Some(host) => format!("http://{}/{}", host, filename),
    None => format!("http://s3.amazonaws.com/{}/{}", config.aws_bucket.clone().unwrap(), filename)
  }
}

pub fn gen_file(dir: String, ext: &str, len: usize) -> PathBuf {
  let filename = gen_filename(ext, len);
  let file_path = Path::new(&dir);
  file_path.join(&filename)
}

pub fn gen_filename_from_existing(file: &Path, strategy: String, len: usize) -> String {
  let file_base = path_file_stem(file);
  let file_ext = path_file_ext(file);
  if (file_ext.is_some()) {
    match strategy.as_ref() {
      "exact" => format!("{}.{}", file_base, file_ext.unwrap()),
      "append" => format!("{}-{}.{}", file_base, rand_string(len), file_ext.unwrap()),
      "replace" => format!("{}.{}", rand_string(len), file_ext.unwrap()),
      _ => format!("{}-{}.{}", file_base, rand_string(len), file_ext.unwrap()),
    }
  } else {
    match strategy.as_ref() {
      "exact" => format!("{}", file_base),
      "append" => format!("{}-{}", file_base, rand_string(len)),
      "replace" => format!("{}", rand_string(len)),
      _ => format!("{}-{}", file_base, rand_string(len)),
    }
  }
}

fn gen_filename(ext: &str, len: usize) -> String {
  format!("{}.{}", rand_string(len), ext)
}

fn rand_string(len: usize) -> String {
  rand::thread_rng().gen_ascii_chars().take(len).collect()
}
