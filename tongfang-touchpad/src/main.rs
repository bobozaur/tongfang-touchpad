use clap::{Parser, ValueEnum};
use tongfang_touchpad_lib::{TPadResult, Touchpad, TouchpadState};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    action: TouchpadAction,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum TouchpadAction {
    On,
    Off,
    Toggle,
}

fn main() -> TPadResult<()> {
    let action = Cli::parse().action;
    let mut tpad = Touchpad::new()?;

    let state = match action {
        TouchpadAction::On => TouchpadState::Enabled,
        TouchpadAction::Off => TouchpadState::Disabled,
        TouchpadAction::Toggle => match tpad.touchpad_state()? {
            TouchpadState::Disabled => TouchpadState::Enabled,
            TouchpadState::ButtonsEnabled
            | TouchpadState::SurfaceEnabled
            | TouchpadState::Enabled => TouchpadState::Disabled,
        },
    };

    tpad.set_touchpad_state(state)
}
