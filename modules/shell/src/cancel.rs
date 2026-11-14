//! « Annuler » (D92): one token for each making, from the moment it is asked to its end — as
//! Go's `context`, tokio's `CancellationToken` and .NET's own. Cancelled, or holding its running
//! program, under one lock: whichever comes first, the other sees it, and the program is stopped
//! once, whatever the order.

use std::sync::{Arc, Mutex};

/// A making's own token: cloned to whoever may cancel it and to whoever runs it.
#[derive(Clone, Default)]
pub struct Cancel(Arc<Mutex<State>>);

#[derive(Default)]
struct State {
    cancelled: bool,
    /// How to stop its program, once one runs and until it is stopped.
    stop: Option<Box<dyn FnOnce() + Send>>,
}

impl Cancel {
    /// Cancels the making: its program, if one is held, is stopped now; one held later is stopped
    /// as it is held; one not started yet is never started by whoever asks [`Cancel::cancelled`].
    pub fn cancel(&self) {
        let stop = {
            let mut state = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            state.cancelled = true;
            state.stop.take()
        };
        if let Some(stop) = stop {
            stop();
        }
    }

    /// Whether it was cancelled: then nothing of it starts, and its end is said cancelled.
    pub fn cancelled(&self) -> bool {
        self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).cancelled
    }

    /// Its program, just started: kept to be stopped on a cancel — or stopped at once, cancelled
    /// already between its start and now.
    pub fn hold(&self, stop: impl FnOnce() + Send + 'static) {
        let mut state = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.cancelled {
            drop(state);
            stop();
        } else {
            state.stop = Some(Box::new(stop));
        }
    }

    /// Whether two tokens are the one making's.
    pub fn same(&self, other: &Cancel) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
