use conf::DropConfig;

use std;

#[cfg(target_os = "linux")]
use gtk;
#[cfg(target_os = "linux")]
use libappindicator::{AppIndicator, AppIndicatorStatus};
#[cfg(target_os = "linux")]
use gtk::prelude::*;
#[cfg(target_os = "linux")]
use libc::*;

#[cfg(target_os = "macos")]
use cocoa::base::{nil, YES, selector};
#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApp, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength, NSApplicationActivationPolicyRegular, NSApplication};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSProcessInfo, NSAutoreleasePool, NSString};

#[cfg(target_os = "linux")]
#[allow(unused)]
pub fn wait_for_user_stop(config: &DropConfig) {
  if gtk::init().is_err() {
    println!("Failed to initialize GTK.");
    std::process::exit(1);
  }

  let mut indicator = AppIndicator::new("Drop", "");
  indicator.set_status(AppIndicatorStatus::Active);
  let icon_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icon");
  indicator.set_icon_theme_path(icon_path.to_str().unwrap());
  indicator.set_icon_full("drop", "icon");
  let mut m = gtk::Menu::new();
  let mi = gtk::CheckMenuItem::with_label("End Recording");
  mi.connect_activate(|_| {
    gtk::main_quit();
  });
  m.append(&mi);
  indicator.set_menu(&mut m);
  m.show_all();

  if config.stop_key.is_some() {
    create_stop_keybinding(config.stop_key.clone().unwrap());
  }

  gtk::main();
}

#[cfg(target_os = "linux")]
fn create_stop_keybinding(keybinding: String) {
  init_keybinder();
  bind_key(keybinding.as_ref(), move |_| {
    gtk::main_quit();
  });
}

#[cfg(target_os = "macos")]
pub fn wait_for_user_stop() {
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

// Adapted from: https://github.com/bram209/rs-gtk-3.0-keybinder
#[cfg(target_os = "linux")]
extern {
  fn keybinder_init();
  fn keybinder_bind(keystring: *const c_char, handler: unsafe extern fn(*const c_char, *mut c_void), user_data: *mut c_void) -> c_int;
}

#[cfg(target_os = "linux")]
unsafe extern fn key_handler<F>(keycode: *const c_char, arg: *mut c_void) where F: FnMut(String) {
  let keycode = std::ffi::CStr::from_ptr(keycode).to_str();
  match keycode {
    Ok(keycode) => {
      let closure = arg as *mut F;
      let keycode = keycode.to_owned();
      (*closure)(keycode);
    },
    Err(_) => {
      println!("Utf8 error for {:?}", keycode)
    }
  }
}

#[cfg(target_os = "linux")]
fn init_keybinder() {
  unsafe { keybinder_init(); }
}

#[cfg(target_os = "linux")]
fn bind_key<F: Fn(String)>(hotkey: &str, callback: F) -> bool where F: FnMut(String) {
  let c_msg = std::ffi::CString::new(hotkey).unwrap();

  let cb = &callback as *const _ as *mut c_void;
  unsafe { keybinder_bind(c_msg.as_ptr(), key_handler::<F>, cb) == 1 }
}
