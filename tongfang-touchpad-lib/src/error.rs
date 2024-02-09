use std::io::Error as IoError;

use nix::Error as NixError;
use thiserror::Error as ThisError;
use udev::Device;

use crate::TOUCHPAD_SYSNAME;

pub type TPadResult<T> = std::result::Result<T, TPadError>;

#[derive(Debug, ThisError)]
pub enum TPadError {
    #[error(transparent)]
    Nix(#[from] NixError),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error("invalid touchpad state: {0}")]
    InvalidState(u8),
    #[error("found no device matching {TOUCHPAD_SYSNAME}")]
    NoDevice,
    #[error("no devnode for device: {0:#?}")]
    NoDevNode(Box<Device>),
}
