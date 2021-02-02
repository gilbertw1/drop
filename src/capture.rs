use ui;
use conf::DropConfig;
use util;

use std;
use std::env;
use std::io;
use std::fs;
use std::process::{Command, Child, ExitStatus};
use std::path::Path;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use sys_info;

#[cfg(target_os = "macos")]
use objc::runtime::{Object, Class};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSString;
#[cfg(target_os = "macos")]
use cocoa::base::{nil, YES, NO};

#[cfg(target_os = "macos")]
pub type Id = *mut Object;

#[cfg(target_os = "macos")]
extern {
  fn CGMainDisplayID() -> u32;
  static AVMediaTypeAudio: Id;
}

#[cfg(target_os = "linux")]
pub fn screenshot(out_path: &Path, config: &DropConfig) {
  if config.display_server == "wayland" {
    screenshot_wayland(out_path, config)
  } else {
    screenshot_x11(out_path, config)
  }
}

pub fn screenshot_wayland(out_path: &Path, config: &DropConfig) {
  let slurp_out = run_slurp(config);
  util::wait_delay(config);
  crop_and_save_screenshot_wayland(slurp_out, out_path, config)
}

pub fn screenshot_x11(out_path: &Path, config: &DropConfig) {
  let slop_out = run_slop(config);
  util::wait_delay(config);
  crop_and_save_screenshot_x11(&slop_out, out_path, config);
}


#[cfg(target_os = "linux")]
pub fn screencast(out_path: &Path, config: &DropConfig) {
  if config.display_server == "wayland" {
    screencast_wayland(out_path, config)
  } else {
    screencast_x11(out_path, config)
  }
}

pub fn screencast_x11(out_path: &Path, config: &DropConfig) {
  let slop_out = run_slop(config);
  util::wait_delay(config);
  let process =
    if config.video_format == "gif" {
      start_cropped_screencast_process_gif(&slop_out, out_path, config)
    } else {
      start_cropped_screencast_process(&slop_out, out_path, config)
    };

  ui::wait_for_user_stop(config);
  let result = terminate_ffmpeg(process);

  if config.video_format == "gif" {
    post_process_screencast_gif(out_path, config)
  }

  if result.is_err() {
    println!("ERROR: Failed to record screencast");
    std::process::exit(1);
  }
}

pub fn screencast_wayland(out_path: &Path, config: &DropConfig) {
  let slurp_out = run_slurp(config);
  util::wait_delay(config);
  let process = start_cropped_screencast_process_wayland(slurp_out, out_path, config);

  ui::wait_for_user_stop(config);
  let result = terminate_ffmpeg(process);

  if config.video_format == "gif" {
    post_process_screencast_gif(out_path, config)
  }

  if result.is_err() {
    println!("ERROR: Failed to record screencast");
    std::process::exit(1);
  }
}

#[cfg(target_os = "macos")]
pub fn screenshot(out_path: &Path, config: &DropConfig) {
  let mut cmd = Command::new("screencapture");
  if (config.delay > 0) {
    cmd.args(&["-T", &config.delay.to_string()]);
  }
  cmd.args(&["-s", &out_path.to_string_lossy().into_owned()]);
  let result = util::run_command_and_wait(&mut cmd, "SCREEN CAPTURE", config);

  if !result.success() {
    println!("Cancelling drop, exiting");
    std::process::exit(1);
  }
}

#[cfg(target_os = "macos")]
pub fn crop_and_take_screencast(out_path: &Path, config: &DropConfig) {
  util::wait_delay(config);
  let capture_session = create_and_initiate_macos_caputure_session(out_path, config);
  ui::wait_for_user_stop();
  end_macos_capture_session(capture_session);
}

fn run_slop(config: &DropConfig) -> SlopOutput {
  let result =
    if config.transparent {
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

fn run_slurp(config: &DropConfig) -> String {
  let result = Command::new("slurp").output().unwrap();

  if !result.status.success() {
    println!("Cancelled drop, exiting");
    std::process::exit(1);
  }

  String::from_utf8(result.stdout).unwrap()
}

fn crop_and_save_screenshot_x11(slop_out: &SlopOutput, out_path: &Path, config: &DropConfig) {
  let mut cmd = Command::new("import");
  cmd.args(&["-window", "root",
             "-crop", &slop_out.g,
             &out_path.to_string_lossy().into_owned()]);

  let result = util::run_command_and_wait(&mut cmd, "IMPORT", config);

  if !result.success() {
    println!("ERROR: Failed to take the screenshot");
    std::process::exit(1);
  }
}

fn crop_and_save_screenshot_wayland(slurp_out: String, out_path: &Path, config: &DropConfig) {
  let mut cmd = Command::new("grim");
  cmd.args(&["-g", slurp_out.trim(), &out_path.to_string_lossy().into_owned()]);

  println!("Command: {:?}", cmd);
  let result = util::run_command_and_wait(&mut cmd, "GRIM", config);

  if !result.success() {
    println!("ERROR: Failed to take the screenshot");
    std::process::exit(1);
  }
}

fn start_cropped_screencast_process(slop_out: &SlopOutput, out_path: &Path, config: &DropConfig) -> Child {
  let mut cmd = Command::new("ffmpeg");
  let display = match env::var("DISPLAY") {
    Ok(display) => display,
    Err(_) => ":0".to_string(),
  };
  cmd.args(vec!["-f", "x11grab",
                "-show_region", if config.border { "1" } else { "0" },
                "-draw_mouse", if config.mouse { "1" } else { "0" },
                "-s", &format!("{}x{}", slop_out.w, slop_out.h),
                "-i", &format!("{}.0+{},{}", display, slop_out.x, slop_out.y)]);

  if config.audio_source == "desktop" {
    cmd.args(vec!["-f", "alsa",
                  "-i", "pulse"]);
  } else {
    cmd.args(vec!["-f", "alsa",
                  "-i", "hw:0"]);
  }

  cmd.args(vec![
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
  if !config.audio {
    cmd.arg("-an");
  }

  cmd.arg(&out_path.to_string_lossy().into_owned());
  util::run_command(&mut cmd, "FFMPEG", config)
}

fn start_cropped_screencast_process_wayland(slurp_out: String, out_path: &Path, config: &DropConfig) -> Child {
  let mut cmd = Command::new("wf-recorder");
  cmd.args(&["-g", slurp_out.trim(), "--file", &out_path.to_string_lossy().into_owned()]);
  util::run_command(&mut cmd, "WF-RECORDER", config)
}


fn start_cropped_screencast_process_gif(slop_out: &SlopOutput, out_path: &Path, config: &DropConfig) -> Child {
  let pamfile = &out_path.to_string_lossy().into_owned().replace(".gif", ".pam");
  let mut cmd = Command::new("ffmpeg");
  cmd.args(&["-f", "x11grab",
             "-show_region", if config.border { "1" } else { "0" },
             "-draw_mouse", if config.mouse { "1" } else { "0" },
             "-framerate", "20",
             "-s", &format!("{}x{}", slop_out.w, slop_out.h),
             "-i", &format!(":0.0+{},{}", slop_out.x, slop_out.y),
             "-codec:v", "pam",
             "-f", "rawvideo",
             pamfile]);
  util::run_command(&mut cmd, "FFMPEG", config)
}

fn post_process_screencast_gif(out_path: &Path, config: &DropConfig) {
  let memory_limit = ((sys_info::mem_info().unwrap().avail as f64 * 0.6) as f64) as u64;
  let cache_id = util::rand_string(30);
  let cachedir = Path::new(&config.dir).join(".cache").join(cache_id);
  let pamfile = out_path.to_str().unwrap().replace(".gif", ".pam");
  let mut process_cmd = Command::new("convert");
  println!("{}kiB", memory_limit);
  process_cmd.args(&["-set", "delay", "5",
                     "-limit", "disk", "unlimited",
                     "-limit", "memory", &format!("{}kiB", memory_limit),
                     "-layers", "Optimize",
                     "-define", &format!("registry:temporary-path={}", cachedir.to_str().unwrap()),
                     &pamfile,
                     out_path.to_str().unwrap()]);

  let result = util::run_command_and_wait(&mut process_cmd, "FFMPEG", config);
  fs::remove_file(pamfile);
  fs::remove_dir_all(cachedir);
  if !result.success() {
    println!("ERROR: Failed to generate gif");
    std::process::exit(1);
  }
}

fn terminate_ffmpeg(mut child: Child) -> io::Result<ExitStatus> {
  let child_id = child.id();
  let result = kill(Pid::from_raw(child_id as i32), Signal::SIGTERM);
  if result.is_err() {
    println!("WARNING: Failed to propertly terminate ffmpeg process")
  }
  child.wait()
}


#[cfg(target_os = "macos")]
fn create_and_initiate_macos_caputure_session(out_path: &Path, config: &DropConfig) -> MacOSAVCaptureSession {
  unsafe {
    let AVCaptureSession = Class::get("AVCaptureSession").unwrap();
    let session: Id = msg_send![AVCaptureSession, alloc];
    let session: Id = msg_send![session, init];

    let AVCaptureScreenInput = Class::get("AVCaptureScreenInput").unwrap();
    let input: Id = msg_send![AVCaptureScreenInput, alloc];
    let input: Id = msg_send![input, initWithDisplayID:CGMainDisplayID()];
    let input: Id = msg_send![input, autorelease];
    msg_send![input, setCapturesCursor: if config.mouse { YES } else { NO }];

    let AVCaptureDevice = Class::get("AVCaptureDevice").unwrap();
    let AVCaptureDeviceInput = Class::get("AVCaptureDeviceInput").unwrap();
    let audio_device: Id = msg_send![AVCaptureDevice, defaultDeviceWithMediaType:AVMediaTypeAudio];
    let error: Id = nil;
    let audio_input: Id = msg_send![AVCaptureDeviceInput, deviceInputWithDevice:audio_device error:error];

    let AVCaptureMovieFileOutput = Class::get("AVCaptureMovieFileOutput").unwrap();
    let output: Id = msg_send![AVCaptureMovieFileOutput, alloc];
    let output: Id = msg_send![output, init];
    let output: Id = msg_send![output, autorelease];

    msg_send![session, addInput:input];
    if config.audio {
      msg_send![session, addInput:audio_input];
    }
    msg_send![session, addOutput:output];
    msg_send![session, startRunning];

    let NSUrl = Class::get("NSURL").unwrap();
    let raw_url = to_ns_string(out_path.to_string_lossy().into_owned());
    let dest_url: Id = msg_send![NSUrl, fileURLWithPath:raw_url];
    msg_send![output, startRecordingToOutputFileURL:dest_url recordingDelegate:session];

    MacOSAVCaptureSession {
      session: session,
      input: input,
      output: output
    }
  }
}

#[cfg(target_os = "macos")]
fn end_macos_capture_session(session: MacOSAVCaptureSession) {
  unsafe {
    msg_send![session.output, stopRecording];
    msg_send![session.session, stopRunning];
  }
}

#[cfg(target_os = "macos")]
fn to_ns_string(str: String) -> Id {
  unsafe {
    let value = std::ffi::CString::new(str).unwrap();
    let NSString = Class::get("NSString").unwrap();
    msg_send![NSString, stringWithUTF8String:value.as_ptr()]
  }
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

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct MacOSAVCaptureSession {
  session: Id,
  input: Id,
  output: Id
}
