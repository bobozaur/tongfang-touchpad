use std::sync::{Arc, Mutex};

use env_logger::Builder;
use gio::glib::MainLoop;
use log::LevelFilter;
use tongfang_touchpad_daemon::{setup_touchpad_power_listener, setup_touchpad_settings_listener};
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

    setup_touchpad_settings_listener(touchpad.clone());
    if let Err(e) = setup_touchpad_power_listener(touchpad) {
        log::error!("error setting up power state listener: {e}");
        return;
    }

    MainLoop::new(None, false).run();
}
