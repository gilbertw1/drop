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

  gtk::main();
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
