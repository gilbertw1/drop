#![feature(libc)]
use ui;
use std;
use std::process::{Command, Stdio, Child};
use std::collections::HashMap;
use std::path::Path;
use libc::{kill, SIGTERM};
use std::os::unix::io::{FromRawFd, AsRawFd};


#[cfg(target_os = "macos")]
pub fn crop_and_take_screenshot(out_path: &Path, transparent: bool) {
  let result =
    Command::new("screencapture").args(&["-s", &out_path.to_string_lossy().into_owned()]).output().unwrap();

  if !result.status.success() {
    println!("Cancelling drop, exiting");
    std::process::exit(1);
  }
}

#[cfg(target_os = "linux")]
pub fn crop_and_take_screenshot(out_path: &Path, transparent: bool) {
  let slop_out = run_slop(transparent);
  crop_and_save_screenshot(&slop_out, out_path);
}

#[cfg(target_os = "linux")]
pub fn crop_and_take_screencast(out_path: &Path, video_format: String, audio: bool, transparent: bool) {
  let slop_out = run_slop(transparent);
  let process =
    if video_format == "gif" {
      start_cropped_screencast_process_gif(&slop_out, out_path)
    } else {
      start_cropped_screencast_process(&slop_out, out_path, audio)
    };
  ui::gtk_stop_recording_popup();
  terminate_ffmpeg(process);
}


#[cfg(target_os = "macos")]
pub fn crop_and_take_screencast(out_path: &Path, video_format: String, audio: bool, transparent: bool) {
  println!("Recording video screencast not supported on MacOS yet.");
  std::process::exit(1);
}

fn run_slop(transparent: bool) -> SlopOutput {
  let result =
    if transparent {
      Command::new("slop").args(&["-l", "-c", "0.3,0.4,0.6,0.4", "-f", "%x %y %w %h %g %i"]).output().unwrap()
    } else {
      Command::new("slop").args(&["-b", "5", "-c", "0.3,0.4,0.6,1", "-f", "%x %y %w %h %g %i"]).output().unwrap()
    };

  if !result.status.success() {
    println!("Cancelled drop, exiting");
    std::process::exit(1);
  }

  let output = String::from_utf8(result.stdout).unwrap();
  let split: Vec<&str> = output.trim().split(" ").collect();

  SlopOutput {
    x: split[0].to_string(),
    y: split[1].to_string(),
    w: split[2].to_string(),
    h: split[3].to_string(),
    g: split[4].to_string(),
    id: split[5].to_string(),
    cancel: false,
  }
}

fn crop_and_save_screenshot(slop_out: &SlopOutput, out_path: &Path) {
  Command::new("import")
    .args(&["-window", "root",
            "-crop", &slop_out.g,
            &out_path.to_string_lossy().into_owned()])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn().unwrap().wait();
}

fn start_cropped_screencast_process(slop_out: &SlopOutput, out_path: &Path, audio: bool) -> Child {
  let mut cmd = Command::new("ffmpeg");
  cmd.args(&["-f", "x11grab",
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
             "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2"]);

  if !audio {
    cmd.arg("-an");
  }

  cmd.arg(&out_path.to_string_lossy().into_owned());
  cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap()
}

fn start_cropped_screencast_process_gif(slop_out: &SlopOutput, out_path: &Path) -> Child {
    Command::new("ffmpeg")
    .args(&["-f", "x11grab",
            "-s", &format!("{}x{}", slop_out.w, slop_out.h),
            "-i", &format!(":0.0+{},{}", slop_out.x, slop_out.y),
            &out_path.to_string_lossy().into_owned()])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
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
