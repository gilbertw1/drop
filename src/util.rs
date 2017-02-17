use std::path::PathBuf;
use rand;
use rand::Rng;

use conf::DropConfig;

pub fn path_to_str(path: &PathBuf) -> String {
  path.to_string_lossy().into_owned()
}

pub fn path_file_name(path: &PathBuf) -> String {
  path.file_name().unwrap().to_string_lossy().into_owned()
}

pub fn create_drop_url(config: &DropConfig, filename: String) -> String {
  match config.drop_host.clone() {
    Some(host) => format!("http://{}/{}", host, filename),
    None => format!("http://s3.amazonaws.com/{}/{}", config.aws_bucket.clone().unwrap(), filename)
  }
}

pub fn gen_file(dir: String, ext: &str) -> PathBuf {
  let filename = gen_filename(ext, 10);
  let mut file_path = PathBuf::from(&dir);
  file_path.push(&filename);
  file_path
}

fn gen_filename(ext: &str, len: usize) -> String {
  let rand_filename: String = rand::thread_rng().gen_ascii_chars().take(len).collect();
  format!("{}.{}", rand_filename, ext)
}
