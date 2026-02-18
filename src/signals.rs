use anyhow::{anyhow, Result};
use log::debug;
use signal_hook::iterator::Signals;
use std::{
    sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    thread,
};

use crate::appearance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeEvent {
    Terminated,
    ThemeChanged,
}

pub fn start_signal_handler(mut signals: Signals, wake_tx: Sender<WakeEvent>) {
    let signals_handle = signals.handle();

    thread::spawn(move || {
        for signal in signals.forever() {
            debug!("Received signal: {signal}");
            let _ = wake_tx.send(WakeEvent::Terminated);
            signals_handle.close();
        }
    });
}

pub fn start_appearance_change_handler(wake_tx: Sender<WakeEvent>) {
    appearance::start_appearance_listener(move || {
        let _ = wake_tx.send(WakeEvent::ThemeChanged);
    });
}

pub fn interruptible_sleep(
    duration: std::time::Duration,
    wake_rx: &Receiver<WakeEvent>,
) -> Result<Option<WakeEvent>> {
    match wake_rx.recv_timeout(duration) {
        Ok(event) => Ok(Some(event)),
        Err(RecvTimeoutError::Timeout) => Ok(None),
        Err(err @ RecvTimeoutError::Disconnected) => Err(anyhow!(err)),
    }
}
