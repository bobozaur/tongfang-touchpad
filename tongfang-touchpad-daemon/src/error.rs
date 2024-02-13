use std::io::Error as IoError;

use gio::glib::{
    value::{ValueTypeMismatchError, ValueTypeMismatchOrNoneError},
    Error as GlibError, GString,
};
use thiserror::Error as ThisError;
use tongfang_touchpad_lib::TPadError;

pub type DaemonResult<T> = Result<T, DaemonError>;

#[derive(Debug, ThisError)]
pub enum DaemonError {
    #[error("unknown touchpad state received from dbus: {0}")]
    UnknownDbusState(GString),
    #[error(transparent)]
    TPadError(#[from] TPadError),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error(transparent)]
    Glib(#[from] GlibError),
    #[error("missing value {1} at position {0}")]
    MissingValue(usize, &'static str),
    #[error(transparent)]
    OptionalValueMismatch(#[from] ValueTypeMismatchOrNoneError<ValueTypeMismatchError>),
    #[error(transparent)]
    ValueMismatch2(#[from] ValueTypeMismatchError),
}
