// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // Fix blank window on Linux with certain GPU/Wayland configurations
  #[cfg(target_os = "linux")]
  std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

  port_manager_lib::run();
}
