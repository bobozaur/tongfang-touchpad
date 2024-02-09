use tongfang_touchpad_lib::{TPadResult, Touchpad, TouchpadState};

fn main() -> TPadResult<()> {
    let mut tpad = Touchpad::new()?;
    let state = tpad.touchpad_state()?;
    let new_state = match state {
        TouchpadState::Disabled => TouchpadState::Enabled,
        TouchpadState::ButtonsEnabled | TouchpadState::SurfaceEnabled | TouchpadState::Enabled => {
            TouchpadState::Disabled
        }
    };

    tpad.set_touchpad_state(new_state)
}
