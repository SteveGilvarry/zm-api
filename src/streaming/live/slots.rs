//! The WebRTC session cap (`streaming.webrtc.max_connections`), shared by
//! the WebSocket signalling sessions and WHEP.
//!
//! A session takes a [`WebRtcSlot`] before it starts and gives it back when
//! it ends (the slot's drop). Taking one is a single atomic step, so two
//! sessions starting together can't both slip under the limit.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct WebRtcSlots {
    used: AtomicUsize,
}

/// A held session slot; dropping it frees the slot.
#[derive(Debug)]
pub struct WebRtcSlot(Arc<WebRtcSlots>);

impl WebRtcSlots {
    /// A slot, or `None` when `limit` are in use.
    pub fn try_acquire(self: &Arc<Self>, limit: usize) -> Option<WebRtcSlot> {
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < limit).then_some(n + 1)
            })
            .ok()
            .map(|_| WebRtcSlot(Arc::clone(self)))
    }

    pub fn in_use(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }
}

impl Drop for WebRtcSlot {
    fn drop(&mut self) {
        self.0.used.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_stop_at_the_limit_and_come_back() {
        let slots = Arc::new(WebRtcSlots::default());
        let a = slots.try_acquire(2).expect("first");
        let _b = slots.try_acquire(2).expect("second");
        assert!(slots.try_acquire(2).is_none());
        drop(a);
        assert_eq!(slots.in_use(), 1);
        assert!(slots.try_acquire(2).is_some());
        assert!(Arc::new(WebRtcSlots::default()).try_acquire(0).is_none());
    }

    #[test]
    fn concurrent_starts_never_pass_the_limit() {
        let slots = Arc::new(WebRtcSlots::default());
        let held: Vec<_> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..64).map(|_| s.spawn(|| slots.try_acquire(10))).collect();
            hs.into_iter().filter_map(|h| h.join().unwrap()).collect()
        });
        assert_eq!(held.len(), 10);
        assert_eq!(slots.in_use(), 10);
        drop(held);
        assert_eq!(slots.in_use(), 0);
    }
}
