//! A tiny thread-safe progress indicator for the self-play / eval binaries.
//!
//! Prints a throttled `label: done/total (p%)` line to **stderr** (so stdout stays clean for
//! machine-readable results). Only emits when the integer percentage advances, so it is cheap
//! even when called from many worker threads.

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Progress {
    label: &'static str,
    total: usize,
    done: AtomicUsize,
    last_percent: AtomicUsize,
}

impl Progress {
    pub fn new(label: &'static str, total: usize) -> Self {
        Self {
            label,
            total,
            done: AtomicUsize::new(0),
            last_percent: AtomicUsize::new(0),
        }
    }

    /// Record one completed unit of work and redraw the progress line if the percentage moved.
    pub fn finish_one(&self) {
        let done = self.done.fetch_add(1, Ordering::Relaxed) + 1;
        let percent = done * 100 / self.total.max(1);
        // `fetch_max` returns the previous max; exactly one thread sees a strictly larger value.
        if percent > self.last_percent.fetch_max(percent, Ordering::Relaxed) {
            let mut stderr = std::io::stderr().lock();
            let _ = write!(
                stderr,
                "\r{}: {}/{} ({}%)      ",
                self.label, done, self.total, percent
            );
            if done >= self.total {
                let _ = writeln!(stderr);
            }
            let _ = stderr.flush();
        }
    }
}
