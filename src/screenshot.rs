#![feature(libc)]
use ui;
use std::process::{Command, Stdio, Child};
use std::collections::HashMap;
use std::path::Path;
use libc::{kill, SIGTERM};

pub fn crop_and_take_screenshot(out_path: &Path) -> bool {
  let slop_out = run_slop();
  if slop_out.cancel {
    println!("Cancelled drop, exiting");
    false
  } else {
    crop_and_save_screenshot(&slop_out, out_path);
    true
  }
}

pub fn crop_and_take_screencast(out_path: &Path) -> bool {
  let slop_out = run_slop();
  if slop_out.cancel {
    println!("Cancelled drop, exiting");
    false
  } else {
    println!("Starting ffmpeg process");
    let mut child = start_cropped_screencast_process(&slop_out, out_path);
    println!("Kicking off stop popup");
    ui::gtk_stop_recording_popup();
    println!("Terminating ffmpeg");
    terminate_ffmpeg(child);
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

fn crop_and_save_screenshot(slop_out: &SlopOutput, out_path: &Path) {
  Command::new("import")
    .args(&["-window", "root",
            "-crop", &slop_out.g,
            &out_path.to_string_lossy().into_owned()])
    .spawn().unwrap().wait();
}

fn start_cropped_screencast_process(slop_out: &SlopOutput, out_path: &Path) -> Child {
  Command::new("ffmpeg")
    .args(&["-f", "x11grab",
            "-s", &format!("{}x{}", slop_out.w, slop_out.h),
            "-i", &format!(":0.0+{},{}", slop_out.x, slop_out.y),
            "-f", "alsa",
            "-i", "pulse",
            "-c:v", "libx264",
            "-c:a", "aac",
            "-crf", "23",
            "-preset", "ultrafast",
            "-movflags", "+faststart",
            "-profile:v", "baseline",
            "-level", "3.0",
            "-pix_fmt", "yuv420p",
            "-ac", "2",
            "-strict", "experimental",
            &out_path.to_string_lossy().into_owned()])
    .spawn().unwrap()
}

fn terminate_ffmpeg(mut child: Child) {
  let child_id = child.id();
  unsafe {
    kill(child_id as i32, SIGTERM);
  }
  child.wait();
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
