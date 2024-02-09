mod error;
mod state;

use std::{fs::File, os::fd::AsRawFd, path::Component};

pub use error::{TPadError, TPadResult};
use nix::ioctl_readwrite_buf;
pub use state::TouchpadState;
use udev::{Device, Enumerator};

const SUBSYSTEM: &str = "hidraw";
const TOUCHPAD_SYSNAME: &str = "i2c-UNIW0001:00";
const HID_IOC_MAGIC: u8 = b'H';

const TOUCHPAD_ENABLED_FEATURE: u8 = 7;
const HID_IOC_S_FEATURE: u8 = 6;
const HID_IOC_G_FEATURE: u8 = 7;

ioctl_readwrite_buf!(_touchpad_state, HID_IOC_MAGIC, HID_IOC_G_FEATURE, u8);
ioctl_readwrite_buf!(_touchpad_set_state, HID_IOC_MAGIC, HID_IOC_S_FEATURE, u8);

pub struct Touchpad(File);

impl Touchpad {
    pub fn new() -> TPadResult<Self> {
        let mut enumerator = Enumerator::new()?;
        enumerator.match_subsystem(SUBSYSTEM)?;

        let Some(device) = enumerator.scan_devices()?.find(Self::device_matches) else {
            return Err(TPadError::NoDevice);
        };

        let Some(path) = device.devnode() else {
            return Err(TPadError::NoDevNode(Box::new(device)));
        };

        let file = File::open(path)?;
        Ok(Self(file))
    }

    pub fn touchpad_state(&mut self) -> TPadResult<TouchpadState> {
        let fd = self.0.as_raw_fd();
        let mut data = [TOUCHPAD_ENABLED_FEATURE, 0];
        unsafe { _touchpad_state(fd, &mut data) }?;
        data[1].try_into().map_err(From::from)
    }

    pub fn set_touchpad_state(&mut self, state: TouchpadState) -> TPadResult<()> {
        let fd = self.0.as_raw_fd();
        let mut data = [TOUCHPAD_ENABLED_FEATURE, state as u8];
        unsafe { _touchpad_set_state(fd, &mut data) }?;
        Ok(())
    }

    fn path_component_matches(comp: Component<'_>) -> bool {
        comp.as_os_str() == TOUCHPAD_SYSNAME
    }

    fn device_matches(device: &Device) -> bool {
        device
            .syspath()
            .components()
            .any(Self::path_component_matches)
    }
}
