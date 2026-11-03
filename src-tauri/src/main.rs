// Lumaq is a desktop widget: never attach a console window, not even in debug.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    lumaq_lib::run();
}
