use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, Mutex,
};

use gio::{
    glib::{object::ObjectExt, Value, Variant},
    BusType, Cancellable, DBusProxy, DBusProxyFlags,
};
use tongfang_touchpad_lib::Touchpad;

use crate::error::{DaemonError, DaemonResult};

const PREPARE_FOR_SLEEP_SIGNAL: &str = "PrepareForSleep";

const LOGIN1_SERVICE_NAME: &str = "org.freedesktop.login1";
const LOGIN1_OBJECT_PATH: &str = "/org/freedesktop/login1";
const LOGIN1_MANAGER_INTERFACE_NAME: &str = "org.freedesktop.login1.Manager";

#[derive(Debug)]
pub struct TouchpadResumer(DBusProxy);

impl TouchpadResumer {
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::missing_errors_doc)]
    pub fn new(touchpad: Arc<Mutex<Touchpad>>) -> DaemonResult<Self> {
        let state = touchpad.lock().unwrap().touchpad_state()? as u8;
        let state = AtomicU8::new(state);

        let dbus_proxy = DBusProxy::for_bus_sync(
            BusType::System,
            DBusProxyFlags::NONE,
            None,
            LOGIN1_SERVICE_NAME,
            LOGIN1_OBJECT_PATH,
            LOGIN1_MANAGER_INTERFACE_NAME,
            Cancellable::NONE,
        )?;

        dbus_proxy.connect("g-signal", true, move |values| {
            let mut touchpad = touchpad.lock().unwrap();
            if let Err(e) = Self::handle_sleep_signal(&mut touchpad, &state, values) {
                log::error!("error handling power state signal: {e}");
            }
            None
        });

        Ok(Self(dbus_proxy))
    }

    fn handle_sleep_signal(
        touchpad: &mut Touchpad,
        state_store: &AtomicU8,
        values: &[Value],
    ) -> DaemonResult<()> {
        let signal_name = Self::extract_signal_name(values)?;

        if signal_name != PREPARE_FOR_SLEEP_SIGNAL {
            return Ok(());
        }

        if Self::extract_arg(values)? {
            Self::store_power_state(touchpad, state_store)
        } else {
            Self::load_power_state(touchpad, state_store)
        }
    }

    fn load_power_state(touchpad: &mut Touchpad, state_store: &AtomicU8) -> DaemonResult<()> {
        let state = state_store.load(Ordering::Acquire).try_into()?;
        println!("restoring touchpad state after wake: {state:?}");
        touchpad.set_touchpad_state(state).map_err(From::from)
    }

    fn store_power_state(touchpad: &mut Touchpad, state_store: &AtomicU8) -> DaemonResult<()> {
        let current_state = touchpad.touchpad_state()?;
        log::info!("storing touchpad state before sleep: {current_state:?}");
        let current_state = current_state as u8;
        state_store.store(current_state, Ordering::Release);
        Ok(())
    }

    fn extract_signal_name(values: &[Value]) -> DaemonResult<&str> {
        let pos = 2;

        let Some(value) = values.get(pos) else {
            return Err(DaemonError::MissingValue(pos, "signal name"));
        };

        value.get().map_err(From::from)
    }

    fn extract_arg(values: &[Value]) -> DaemonResult<bool> {
        let pos = 3;

        let Some(value) = values.get(pos) else {
            return Err(DaemonError::MissingValue(pos, "signal argument"));
        };

        let args: Variant = value.get()?;
        Ok(args.child_get(0))
    }
}
