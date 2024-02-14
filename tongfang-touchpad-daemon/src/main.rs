use std::sync::{Arc, Mutex};

use env_logger::Builder;
use gio::glib::MainLoop;
use log::LevelFilter;
use tongfang_touchpad_daemon::{TouchpadResumer, TouchpadSettings};
use tongfang_touchpad_lib::Touchpad;

fn main() {
    Builder::new().filter_level(LevelFilter::Info).init();

    let touchpad = match Touchpad::new() {
        Ok(tpad) => Arc::new(Mutex::new(tpad)),
        Err(e) => {
            log::error!("error setting up touchpad: {e}");
            return;
        }
    };

    let _settings = TouchpadSettings::new(touchpad.clone());
    let _resumer = match TouchpadResumer::new(touchpad) {
        Ok(v) => v,
        Err(e) => {
            log::error!("error setting up power state listener: {e}");
            return;
        }
    };

    MainLoop::new(None, false).run();
}
