use ui;
use conf::DropConfig;
use util;

use std;
use std::env;
use std::io;
use std::process::{Command, Child, ExitStatus};
use std::path::Path;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

#[cfg(target_os = "macos")]
use objc::runtime::{Object, Class, BOOL, YES, NO, Sel};
#[cfg(target_os = "macos")]
use objc::declare::ClassDecl;
#[cfg(target_os = "macos")]
use cocoa::base::{selector, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSString;

#[cfg(target_os = "macos")]
pub type Id = *mut Object;

#[cfg(target_os = "macos")]
extern {
  fn CGMainDisplayID() -> u32;
}

#[cfg(target_os = "linux")]
pub fn crop_and_take_screenshot(out_path: &Path, config: &DropConfig) {
  let slop_out = run_slop(config);
  crop_and_save_screenshot(&slop_out, out_path, config);
}

#[cfg(target_os = "linux")]
pub fn crop_and_take_screencast(out_path: &Path, config: &DropConfig) {
  let slop_out = run_slop(config);
  let process =
    if config.video_format == "gif" {
      start_cropped_screencast_process_gif(&slop_out, out_path, config)
    } else {
      start_cropped_screencast_process(&slop_out, out_path, config)
    };

  println!("STARTED FFMPEG COMMAND");
  println!("WAITING FOR USER STOP");
  ui::wait_for_user_stop(config);
  println!("FINISHED USER STOP");
  let result = terminate_ffmpeg(process);

  if result.is_err() {
    println!("ERROR: Failed to record screencast");
    std::process::exit(1);
  }
}

#[cfg(target_os = "macos")]
pub fn crop_and_take_screenshot(out_path: &Path, config: &DropConfig) {
  let mut cmd = Command::new("screencapture").args(&["-s", &out_path.to_string_lossy().into_owned()]);
  let result = util::run_command_and_wait(&mut cmd, "SCREEN CAPTURE", config);

  if !result.success() {
    println!("Cancelling drop, exiting");
    std::process::exit(1);
  }
}

#[cfg(target_os = "macos")]
pub fn crop_and_take_screencast(out_path: &Path, config: &DropConfig) {
  let capture_session = create_and_initiate_macos_caputure_session(out_path, config.audio);
  wait_for_user_stop();
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

fn crop_and_save_screenshot(slop_out: &SlopOutput, out_path: &Path, config: &DropConfig) {
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

fn start_cropped_screencast_process(slop_out: &SlopOutput, out_path: &Path, config: &DropConfig) -> Child {
  let mut cmd = Command::new("ffmpeg");
  let display = match env::var("DISPLAY") {
    Ok(display) => display,
    Err(_) => ":0".to_string(),
  };
  cmd.args(&["-f", "x11grab",
             "-s", &format!("{}x{}", slop_out.w, slop_out.h),
             "-i", &format!("{}.0+{},{}", display, slop_out.x, slop_out.y),
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

  if !config.audio {
    cmd.arg("-an");
  }

  cmd.arg(&out_path.to_string_lossy().into_owned());
  println!("STARTING FFMPEG COMMAND");
  util::run_command(&mut cmd, "FFMPEG", config)
}

fn start_cropped_screencast_process_gif(slop_out: &SlopOutput, out_path: &Path, config: &DropConfig) -> Child {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(&["-f", "x11grab",
            "-s", &format!("{}x{}", slop_out.w, slop_out.h),
            "-i", &format!(":0.0+{},{}", slop_out.x, slop_out.y),
            &out_path.to_string_lossy().into_owned()]);
  util::run_command(&mut cmd, "FFMPEG", config)
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
fn create_and_initiate_macos_caputure_session(out_path: &Path, audio: bool) -> MacOSAVCaptureSession {
  unsafe {
    let AVCaptureSession = Class::get("AVCaptureSession").unwrap();
    let session: Id = msg_send![AVCaptureSession, alloc];
    let session: Id = msg_send![session, init];

    let AVCaptureScreenInput = Class::get("AVCaptureScreenInput").unwrap();
    let input: Id = msg_send![AVCaptureScreenInput, alloc];
    let input: Id = msg_send![input, initWithDisplayID: CGMainDisplayID()];
    let input: Id = msg_send![input, autorelease];

    let AVCaptureMovieFileOutput = Class::get("AVCaptureMovieFileOutput").unwrap();
    let output: Id = msg_send![AVCaptureMovieFileOutput, alloc];
    let output: Id = msg_send![output, init];
    let output: Id = msg_send![output, autorelease];

    msg_send![session, addInput:input];
    msg_send![session, addOutput:output];
    msg_send![session, startRunning];

    let NSUrl = Class::get("NSURL").unwrap();
    let rawUrl = to_ns_string(out_path.to_string_lossy().into_owned());
    let destUrl: Id = msg_send![NSUrl, fileURLWithPath:rawUrl];
    msg_send![output, startRecordingToOutputFileURL:destUrl recordingDelegate:session];

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
    let value = CString::new(str).unwrap();
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
