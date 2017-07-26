use std;
#[cfg(target_os = "linux")]
use gtk;
#[cfg(target_os = "linux")]
use gtk::prelude::*;

#[cfg(target_os = "linux")]
pub fn gtk_stop_recording_popup() {
  if gtk::init().is_err() {
    println!("Failed to initialize GTK.");
    std::process::exit(1);
  }

  let window = gtk::Window::new(gtk::WindowType::Popup);

  window.set_title("Drop Screen Record");
  window.set_border_width(10);
  window.set_position(gtk::WindowPosition::None);

  window.connect_delete_event(|_, _| {
    gtk::main_quit();
    Inhibit(false)
  });

  let button = gtk::Button::new_with_label("Stop Recording");

  window.add(&button);
  let wclone = window.clone();

  button.connect_clicked(move |_| {
    wclone.destroy();
    gtk::main_quit();
  });

  window.show_all();
  gtk::main();
}
