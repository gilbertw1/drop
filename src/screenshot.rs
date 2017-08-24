use ui;
use std;
use std::process::{Command, Stdio, Child};
use std::path::Path;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use conf::DropConfig;

#[cfg(target_os = "macos")]
use objc::runtime::{Object, Class, BOOL, YES, NO, Sel};
#[cfg(target_os = "macos")]
use objc::declare::ClassDecl;
#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApp, NSApplication, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength, NSApplicationActivationPolicyRegular};
#[cfg(target_os = "macos")]
use cocoa::base::{selector, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSProcessInfo, NSAutoreleasePool, NSString};

#[cfg(target_os = "macos")]
pub type Id = *mut Object;

#[cfg(target_os = "macos")]
extern {
  fn CGMainDisplayID() -> u32;
}

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
pub fn crop_and_take_screencast(out_path: &Path, video_format: String, config: &DropConfig) {
  let slop_out = run_slop(config.transparent);
  let process =
    if video_format == "gif" {
      start_cropped_screencast_process_gif(&slop_out, out_path, config.verbose)
    } else {
      start_cropped_screencast_process(&slop_out, out_path, config.audio, config.verbose)
    };
  ui::gtk_stop_recording_popup();
  terminate_ffmpeg(process);
}


#[cfg(target_os = "macos")]
pub fn crop_and_take_screencast(out_path: &Path, video_format: String, audio: bool, transparent: bool) {
  let capture_session = create_and_initiate_macos_caputure_session(out_path, audio);
  create_status_bar_menu_and_wait_for_stop();
  end_macos_capture_session(capture_session);
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
fn create_status_bar_menu_and_wait_for_stop() {
  unsafe {
    let _pool = NSAutoreleasePool::new(nil);
    let app = NSApp();
    app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

    let sbar = NSStatusBar::systemStatusBar(nil);

    let sbar_item = sbar.statusItemWithLength_(NSVariableStatusItemLength);
    msg_send![sbar_item.button(), setTitle:NSString::alloc(nil).init_str("DROP")];
    msg_send![sbar_item.button(), setHighlighted:YES];

    let sbar_menu = NSMenu::new(nil).autorelease();
    let stop_prefix = NSString::alloc(nil).init_str("Stop Recording");
    let stop_title = stop_prefix.stringByAppendingString_(NSProcessInfo::processInfo(nil).processName());
    let stop_action = selector("stop:");
    let stop_key = NSString::alloc(nil).init_str("q");
    let stop_item = NSMenuItem::alloc(nil)
      .initWithTitle_action_keyEquivalent_(stop_title, stop_action, stop_key)
      .autorelease();

    sbar_menu.addItem_(stop_item);
    sbar_item.setMenu_(sbar_menu);

    app.run();
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

fn start_cropped_screencast_process(slop_out: &SlopOutput, out_path: &Path, audio: bool, verbose: bool) -> Child {
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

  if verbose {
    cmd.spawn().unwrap()
  } else {
    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap()
  }
}

fn start_cropped_screencast_process_gif(slop_out: &SlopOutput, out_path: &Path, verbose: bool) -> Child {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(&["-f", "x11grab",
            "-s", &format!("{}x{}", slop_out.w, slop_out.h),
            "-i", &format!(":0.0+{},{}", slop_out.x, slop_out.y),
            &out_path.to_string_lossy().into_owned()]);
    if verbose {
      cmd.spawn().unwrap()
    } else {
      cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap()
    }
}

fn terminate_ffmpeg(mut child: Child) {
  let child_id = child.id();
  kill(Pid::from_raw(child_id as i32), Signal::SIGTERM);
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

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct MacOSAVCaptureSession {
  session: Id,
  input: Id,
  output: Id
}
