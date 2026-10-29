//! Recovery for event taps that macOS disables at runtime.
//!
//! macOS disables a tap whose callback runs too long (`TapDisabledByTimeout`)
//! or when user input requires it (`TapDisabledByUserInput`, e.g. secure input).
//! A disabled tap stays dead until someone calls `CGEventTapEnable` again, so
//! without this the listener silently stops delivering events for the rest of
//! the process lifetime.

use objc2_core_foundation::CFMachPort;
use objc2_core_graphics::{CGEvent, CGEventType};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

/// The running tap for one entry point (listen or grab).
pub(crate) struct TapSlot(AtomicPtr<CFMachPort>);

impl TapSlot {
    pub(crate) const fn new() -> Self {
        Self(AtomicPtr::new(null_mut()))
    }

    /// Records the tap so the callback can re-enable it. The caller keeps `tap`
    /// alive for as long as its run loop runs, and calls `clear` before dropping it.
    pub(crate) fn set(&self, tap: &CFMachPort) {
        self.0.store(
            tap as *const CFMachPort as *mut CFMachPort,
            Ordering::SeqCst,
        );
    }

    pub(crate) fn clear(&self) {
        self.0.store(null_mut(), Ordering::SeqCst);
    }

    /// Re-enables the tap if `event_type` is a disable notice. Returns whether it was one;
    /// disable notices carry no input and must not be converted into events.
    pub(crate) fn reenable_if_disabled(&self, event_type: CGEventType) -> bool {
        let reason = match event_type {
            CGEventType::TapDisabledByTimeout => &REENABLED_AFTER_TIMEOUT,
            CGEventType::TapDisabledByUserInput => &REENABLED_AFTER_USER_INPUT,
            _ => return false,
        };
        let tap = self.0.load(Ordering::SeqCst);
        if !tap.is_null() {
            // SAFETY: `set` stored a tap its owner keeps alive while the run loop
            // delivering this callback runs; `clear` runs before it is dropped.
            CGEvent::tap_enable(unsafe { &*tap }, true);
        }
        reason.fetch_add(1, Ordering::SeqCst);
        true
    }
}

static REENABLED_AFTER_TIMEOUT: AtomicU64 = AtomicU64::new(0);
static REENABLED_AFTER_USER_INPUT: AtomicU64 = AtomicU64::new(0);

/// How many times macOS disabled a listen/grab tap and rdev re-enabled it,
/// since process start. Events that arrived while the tap was disabled are
/// lost, so callers that need complete input should compare this before and
/// after a capture and report any increase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TapReenableCounts {
    /// The tap callback took too long (macOS's tap timeout).
    pub after_timeout: u64,
    /// macOS disabled the tap because of user input, such as secure text entry.
    pub after_user_input: u64,
}

impl TapReenableCounts {
    pub fn total(&self) -> u64 {
        self.after_timeout + self.after_user_input
    }
}

pub fn tap_reenable_counts() -> TapReenableCounts {
    TapReenableCounts {
        after_timeout: REENABLED_AFTER_TIMEOUT.load(Ordering::SeqCst),
        after_user_input: REENABLED_AFTER_USER_INPUT.load(Ordering::SeqCst),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_disable_notices_are_handled() {
        let slot = TapSlot::new();
        let before = tap_reenable_counts();
        assert!(!slot.reenable_if_disabled(CGEventType::KeyDown));
        assert_eq!(tap_reenable_counts(), before);

        assert!(slot.reenable_if_disabled(CGEventType::TapDisabledByTimeout));
        assert!(slot.reenable_if_disabled(CGEventType::TapDisabledByUserInput));
        let after = tap_reenable_counts();
        assert!(after.after_timeout > before.after_timeout);
        assert!(after.after_user_input > before.after_user_input);
    }
}
