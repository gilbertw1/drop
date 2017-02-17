use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn crop_and_take_screenshot(out_path: &PathBuf) -> bool {
  let slop_out = run_slop();
  if slop_out.cancel {
    println!("Cancelled drop, exiting");
    false
  } else {
    crop_and_save_screenshot(&slop_out, &out_path);
    true
  }
}

fn run_slop() -> SlopOutput {
  let result = Command::new("slop").args(&["--color", "0.275,0.510,0.706"]).output().unwrap();
  let output = String::from_utf8(result.stdout).unwrap();
  let out_map = output.trim()
    .split("\n")
    .map(|kv| kv.split("="))
    .map(|mut kv| (kv.next().unwrap().into(), kv.next().unwrap().into()))
    .collect::<HashMap<String, String>>();

  SlopOutput {
    x: out_map.get("X").unwrap().clone(),
    y: out_map.get("Y").unwrap().clone(),
    w: out_map.get("W").unwrap().clone(),
    h: out_map.get("H").unwrap().clone(),
    g: out_map.get("G").unwrap().clone(),
    id: out_map.get("ID").unwrap().clone(),
    cancel: out_map.get("Cancel").unwrap().clone() == "true",
  }
}

fn crop_and_save_screenshot(slop_out: &SlopOutput, out_path: &PathBuf) {
  Command::new("import")
    .args(&["-window", "root", "-crop", &slop_out.g, &out_path.to_string_lossy().into_owned()])
    .spawn().unwrap().wait();
}

#[derive(Debug)]
struct SlopOutput {
  x: String,
  y: String,
  w: String,
  h: String,
  g: String,
  id: String,
  cancel: bool,
}
