use ui;
use std;
use std::process::{Command, Stdio, Child};
use std::path::Path;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

#[cfg(target_os = "macos")]
use objc::runtime::{Object, Class, BOOL, YES, NO, Sel};
#[cfg(target_os = "macos")]
use objc::declare::ClassDecl;
#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApp, NSApplication, NSView, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength, NSApplicationActivationPolicyRegular};
#[cfg(target_os = "macos")]
use cocoa::base::{selector, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSProcessInfo, NSAutoreleasePool, NSString};
use std::ffi::CString;

#[cfg(target_os = "macos")]
pub type Id = *mut Object;

#[cfg(target_os = "macos")]
extern {
  fn fabs(x: f64) -> f64;
  fn min(x: f64, y: f64) -> f64;
  fn CGMainDisplayID() -> u32;
  fn CGRectMake(x: f64, y: f64, width: f64, height: f64) -> Id;
  fn CGPathCreateMutable() -> Id;
  fn CGPathMoveToPoint(path: Id, transform: Id, x: f64, y: f64);
  fn CGPathAddLineToPoint(path: Id, transform: Id, x: f64, y: f64);
  fn CGPathCloseSubpath(path: Id);
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
  output: Id,
}

#[cfg(target_os = "macos")]
struct CroppingView {
  super_: cocoa::NSView,
  clicked_point: Id,
  shape_layer: Id,
}

#[cfg(target_os = "macos")]
impl cocoa::Object for CroppingView {
  type Super = cocoa::NSView;

  fn super_ref(&self) -> &Self::Super {
    &self.super_
  }
}

#[cfg(target_os = "macos")]
fn register_macos_cropping_view() {
  unsafe {
    let NSObject = Class::get("NSView").unwrap();
    let mut decl = ClassDecl::new("CroppingView", NSObject).unwrap();
    decl.add_ivar::<Id>("_clickedPoint");
    decl.add_ivar::<Id>("_shapeLayer");

    extern fn impl_init(self_: &mut AnyObject, sel: Sel) {
      unsafe {
        let CALayer = Class::get("CALayer").unwrap();
        let layer: Id = msg_send![CALayer, alloc];
        let layer: Id = msg_send![layer, init];

        let super_view = NSView::init();
        super_view.setLayer(layer);
        super_view.setWantsLayer(YES);

        let NSColor = Class::get("NSColor").unwrap();
        let black_color: Id = msg_send![NSColor, blackColor];
        let black_cgcolor: Id = msg_send![black_color, CGColor];
        let clear_color: Id = msg_send![NSColor, clearColor];
        let clear_cgcolor: Id = msg_send![clear_color, CGColor];

        let CAShapeLayer = Class::get("CAShapeLayer").unwrap();
        let shape_layer: Id = msg_send![CAShapeLayer, alloc];
        let shape_layer: Id = msg_send![shape_layer, init];
        msg_send![shape_layer, setLineWidth:1.0 as f64];
        msg_send![shape_layer, setStrokeColor: black_cgcolor];
        msg_send![shape_layer, setFillColor: clear_cgcolor];
      }
    }
  }
}

#[cfg(target_os = "macos")]
impl CroppingView {
  fn new() -> Self {
    unsafe {
      let CALayer = Class::get("CALayer").unwrap();
      let layer: Id = msg_send![CALayer, alloc];
      let layer: Id = msg_send![layer, init];

      let super_view = NSView::init();
      super_view.setLayer(layer);
      super_view.setWantsLayer(YES);

      let NSColor = Class::get("NSColor").unwrap();
      let black_color: Id = msg_send![NSColor, blackColor];
      let black_cgcolor: Id = msg_send![black_color, CGColor];
      let clear_color: Id = msg_send![NSColor, clearColor];
      let clear_cgcolor: Id = msg_send![clear_color, CGColor];

      let CAShapeLayer = Class::get("CAShapeLayer").unwrap();
      let shape_layer: Id = msg_send![CAShapeLayer, alloc];
      let shape_layer: Id = msg_send![shape_layer, init];
      msg_send![shape_layer, setLineWidth:1.0 as f64];
      msg_send![shape_layer, setStrokeColor: black_cgcolor];
      msg_send![shape_layer, setFillColor: clear_cgcolor];

      CroppingView {
        super_: super_view,
        clicked_point: nil,
        shape_layer: shape_layer
      }
    }
  }

  fn mouse_down(&self, event: Id) {
    unsafe {
      let location_in_window: Id = msg_send![event, locationInWindow];
      let clicked_point: Id = msg_send![self.super_, convertPoint:location_in_window fromView: nil];
      let layer = self.super_.layer();
      msg_send![layer, addSublayer:self.shape_layer];
    }
  }

  fn mouse_up(&self, event: Id) {
    unsafe {
      let location_in_window: Id = msg_send![event, locationInWindow];
      let point: Id = msg_send![self.super_, convertPoint:location_in_window fromView: nil];
      let point_x: f64 = msg_send![point, x];
      let point_y: f64 = msg_send![point, y];
      let clicked_point: Id = self.clicked_point;
      let clicked_point_x: f64 = msg_send![clicked_point, x];
      let clicked_point_y: f64 = msg_send![clicked_point, y];
      let x = min(point_x, clicked_point_x);
      let y = min(point_y, clicked_point_y);
      let width = fabs(point_x - clicked_point_x);
      let height = fabs(point_y - clicked_point_y);
      let selected_rect = CGRectMake(x, y, width, height);
      // TODO: GET THIS VALUE OUT! (selected_rect)
    }
  }

  fn mouse_dragged(&self, event: Id) {
    unsafe {
      let point: Id = msg_send![self.super_, convertPoint:location_in_window fromView: nil];
      let point_x: f64 = msg_send![point, x];
      let point_y: f64 = msg_send![point, y];
      let clicked_point: Id = self.clicked_point;
      let clicked_point_x: f64 = msg_send![clicked_point, x];
      let clicked_point_y: f64 = msg_send![clicked_point, y];

      let path = CGPathCreateMutable();
      CGPathMoveToPoint(path, nil, clicked_point_x, clicked_point_y);
      CGPathAddLineToPoint(path, nil, clicked_point_x, point_y);
      CGPathAddLineToPoint(path, nil, point_x, point_y);
      CGPathAddLineToPoint(path, nil, point_x, clicked_point_y);
      CGPathCloseSubpath(path);

      msg_send![self.shape_layer, setPath: path];
    }
  }
}

fn register_cropping_view() {
  let NSView = Class::get("NSView").unwrap();
}

#[cfg(target_os = "macos")]
struct CroppingWindow {
  super_: cocoa::NSWindow
}

#[cfg(target_os = "macos")]
impl cocoa::Object for CroppingWindow {
  type Super = cocoa::NSWindow;

  fn super_ref(&self) -> &Self::Super {
    &self.super_
  }
}

#[cfg(target_os = "macos")]
impl CroppingWindow {
  fn new() -> Self {
    
  }
}
