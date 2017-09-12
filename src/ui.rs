use std;

#[cfg(target_os = "linux")]
use gtk;
#[cfg(target_os = "linux")]
use gtk::prelude::*;

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApp, NSMenu, NSMenuItem, NSStatusBar, NSVariableStatusItemLength, NSApplicationActivationPolicyRegular};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSProcessInfo, NSAutoreleasePool, NSString};

#[cfg(target_os = "linux")]
pub fn gtk_create_status_icon_and_wait_for_stop() {
  if gtk::init().is_err() {
    println!("Failed to initialize GTK.");
    std::process::exit(1);
  }

  let status_icon = gtk::StatusIcon::new_from_icon_name("camera");
  status_icon.set_title("Drop");

  status_icon.connect_activate(|icon| {

    let menu = gtk::Menu::new();
    let stop_recording = gtk::MenuItem::new_with_label("Stop Recording");
    menu.append(&stop_recording);
    let iclone = icon.clone();

    stop_recording.connect_activate(move |_| {
      iclone.set_visible(false);
      gtk::main_quit();
    });

    stop_recording.show();
    let button: u32 = 0;
    menu.popup_easy(button, gtk::get_current_event_time());
  });

  gtk::main();
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
