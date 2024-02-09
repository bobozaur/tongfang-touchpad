use crate::TPadError;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum TouchpadState {
    Disabled = 0,
    ButtonsEnabled = 1,
    SurfaceEnabled = 2,
    Enabled = 3,
}

impl TryFrom<u8> for TouchpadState {
    type Error = TPadError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let output = match value {
            0 => TouchpadState::Disabled,
            1 => TouchpadState::ButtonsEnabled,
            2 => TouchpadState::SurfaceEnabled,
            3 => TouchpadState::Enabled,
            v => return Err(TPadError::InvalidState(v)),
        };

        Ok(output)
    }
}
