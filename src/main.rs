use std::{error::Error, fs::File, ops::Not, os::fd::AsRawFd, path::Component};

use nix::ioctl_readwrite_buf;
use udev::{Device, Enumerator};

const SUBSYSTEM: &str = "hidraw";
const TOUCHPAD_SYSNAME: &str = "i2c-UNIW0001:00";
const HID_IOC_MAGIC: u8 = b'H';

const TOUCHPAD_ENABLED_FEATURE: u8 = 7;
const HID_IOC_S_FEATURE: u8 = 6;
const HID_IOC_G_FEATURE: u8 = 7;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum TouchpadState {
    Disabled = 0,
    // ButtonsEnabled = 1,
    // SurfaceEnabled = 2,
    Enabled = 3,
}

impl TryFrom<u8> for TouchpadState {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let output = match value {
            0 => TouchpadState::Disabled,
            // 1 => TouchpadState::ButtonsEnabled,
            // 2 => TouchpadState::SurfaceEnabled,
            3 => TouchpadState::Enabled,
            v => return Err(format!("invalid touchpad state {v}")),
        };

        Ok(output)
    }
}

impl Not for TouchpadState {
    type Output = TouchpadState;

    fn not(self) -> Self::Output {
        match self {
            TouchpadState::Disabled => TouchpadState::Enabled,
            TouchpadState::Enabled => TouchpadState::Disabled,
        }
    }
}

ioctl_readwrite_buf!(_touchpad_state, HID_IOC_MAGIC, HID_IOC_G_FEATURE, u8);
ioctl_readwrite_buf!(_touchpad_set_state, HID_IOC_MAGIC, HID_IOC_S_FEATURE, u8);

fn path_component_matches(comp: Component<'_>) -> bool {
    comp.as_os_str() == TOUCHPAD_SYSNAME
}

fn device_matches(device: &Device) -> bool {
    device.syspath().components().any(path_component_matches)
}

fn touchpad_state(file: &mut File) -> Result<TouchpadState, Box<dyn Error>> {
    let fd = file.as_raw_fd();
    let mut data = [TOUCHPAD_ENABLED_FEATURE, 0];
    unsafe { _touchpad_state(fd, &mut data) }?;
    data[1].try_into().map_err(From::from)
}

fn set_touchpad_state(file: &mut File, state: TouchpadState) -> Result<(), Box<dyn Error>> {
    let fd = file.as_raw_fd();
    let mut data = [TOUCHPAD_ENABLED_FEATURE, state as u8];
    unsafe { _touchpad_set_state(fd, &mut data) }?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut enumerator = Enumerator::new()?;
    enumerator.match_subsystem(SUBSYSTEM)?;

    let Some(device) = enumerator.scan_devices()?.find(device_matches) else {
        let msg = format!("no touchpad device matching {TOUCHPAD_SYSNAME} found");
        return Err(msg.into());
    };

    let Some(path) = device.devnode() else {
        let msg = format!("no devnode found for device {device:#?}");
        return Err(msg.into());
    };

    let mut file = File::open(path)?;
    let state = touchpad_state(&mut file)?;
    set_touchpad_state(&mut file, !state)?;
    Ok(())
}
