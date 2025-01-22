use anyhow::{anyhow, Result};
use log::debug;
use signal_hook::iterator::Signals;
use std::{
    sync::mpsc::{channel, Receiver, RecvTimeoutError},
    thread,
};

pub fn start_signal_handler(mut signals: Signals) -> Receiver<bool> {
    let signals_handle = signals.handle();

    let (termination_tx, termination_rx) = channel::<bool>();

    thread::spawn(move || {
        for signal in signals.forever() {
            debug!("Received signal: {}", signal);
            termination_tx.send(true).unwrap();
            signals_handle.close();
        }
    });

    termination_rx
}

pub fn interruptible_sleep(
    duration: std::time::Duration,
    interrupt_rx: &Receiver<bool>,
) -> Result<bool> {
    match interrupt_rx.recv_timeout(duration) {
        Ok(_) => Ok(true),
        Err(RecvTimeoutError::Timeout) => Ok(false),
        Err(err @ RecvTimeoutError::Disconnected) => Err(anyhow!(err)),
    }
}
