use std::{
    path::Component,
    sync::{Arc, Mutex},
};

use gio::{glib::GString, prelude::SettingsExt, Settings};
use tongfang_touchpad_lib::{Touchpad, TouchpadState};
use udev::{Device, Enumerator};

use crate::error::{DaemonError, DaemonResult};

const TOUCHPAD_SETTINGS_SCHEMA_ID: &str = "org.cinnamon.desktop.peripherals.touchpad";
const SEND_EVENTS_SIGNAL: &str = "send-events";
const MOUSE_DEV_PREFIX: &str = "mouse";
const INPUT_SUBSYSTEM: &str = "input";

#[derive(Debug)]
pub struct TouchpadSettings(Settings);

impl TouchpadSettings {
    #[allow(clippy::missing_panics_doc)]
    pub fn new(touchpad: Arc<Mutex<Touchpad>>) -> Self {
        let settings = Settings::new(TOUCHPAD_SETTINGS_SCHEMA_ID);

        settings.connect_changed(Some(SEND_EVENTS_SIGNAL), move |s, e| {
            let mut touchpad = touchpad.lock().unwrap();
            let value = s.string(e);
            if let Err(e) = Self::act_on_changed_settings(&mut touchpad, value) {
                log::error!("error handling touchpad settings changed signal {e}");
            }
        });

        Self(settings)
    }

    fn act_on_changed_settings(touchpad: &mut Touchpad, dbus_value: GString) -> DaemonResult<()> {
        let dbus_state = dbus_value.try_into()?;
        log::info!("handling touchpad DBus state: {dbus_state:?}");

        let state = match dbus_state {
            TouchpadDbusState::Disabled => TouchpadState::Disabled,
            TouchpadDbusState::DisabledOnExternalMouse if Self::is_mouse_connected()? => {
                TouchpadState::Disabled
            }
            TouchpadDbusState::DisabledOnExternalMouse | TouchpadDbusState::Enabled => {
                TouchpadState::Enabled
            }
        };

        touchpad.set_touchpad_state(state)?;
        Ok(())
    }

    fn is_mouse_connected() -> DaemonResult<bool> {
        let mut enumerator = Enumerator::new()?;
        enumerator.match_subsystem(INPUT_SUBSYSTEM)?;
        let out = enumerator.scan_devices()?.any(Self::is_different_mouse);
        Ok(out)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_different_mouse(device: Device) -> bool {
        let Some(devnode) = device.devnode() else {
            return false;
        };

        let is_mouse = devnode
            .components()
            .last()
            .map(Self::is_mouse_device)
            .unwrap_or_default();

        is_mouse && !Touchpad::device_matches(&device)
    }

    fn is_mouse_device(cmp: Component<'_>) -> bool {
        cmp.as_os_str()
            .to_str()
            .map(|s| s.starts_with(MOUSE_DEV_PREFIX))
            .unwrap_or_default()
    }
}

#[derive(Copy, Clone, Debug)]
enum TouchpadDbusState {
    Enabled,
    Disabled,
    DisabledOnExternalMouse,
}

impl TryFrom<GString> for TouchpadDbusState {
    type Error = DaemonError;

    fn try_from(value: GString) -> Result<Self, Self::Error> {
        let out = match value.as_str() {
            "enabled" => Self::Enabled,
            "disabled" => Self::Disabled,
            "disabled-on-external-mouse" => Self::DisabledOnExternalMouse,
            _ => return Err(DaemonError::UnknownDbusState(value)),
        };

        Ok(out)
    }
}
