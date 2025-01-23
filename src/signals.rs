use anyhow::{anyhow, Result};
use log::debug;
use signal_hook::iterator::Signals;
use std::{
    sync::mpsc::{channel, Receiver, RecvTimeoutError},
    thread,
};

pub fn start_signal_handler(mut signals: Signals) -> Receiver<()> {
    let signals_handle = signals.handle();

    let (termination_tx, termination_rx) = channel::<()>();

    thread::spawn(move || {
        for signal in signals.forever() {
            debug!("Received signal: {}", signal);
            termination_tx.send(()).unwrap();
            signals_handle.close();
        }
    });

    termination_rx
}

pub fn interruptible_sleep(
    duration: std::time::Duration,
    interrupt_rx: &Receiver<()>,
) -> Result<bool> {
    match interrupt_rx.recv_timeout(duration) {
        Ok(()) => Ok(true),
        Err(RecvTimeoutError::Timeout) => Ok(false),
        Err(err @ RecvTimeoutError::Disconnected) => Err(anyhow!(err)),
    }
}
