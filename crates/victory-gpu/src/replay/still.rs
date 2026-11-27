//! While the recorder holds the title still (the card's memory copied out,
//! what the CPU wrote compared), the title's sleeps do not count that time:
//! a hang watchdog that counts sleeps (Grand Theft Auto VI's ends the
//! process after 30 `Sleep(1000)` without a frame) sees no more time pass
//! than the frame takes without the recorder. On with `VICTORY_RECORD` only:
//! then the title's `Sleep` and `SleepEx` are bound to the wrappers here
//! ([`import`]).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

struct State {
    /// Holds taken ([`Still`]), and since when the first of them.
    holds: u32,
    since: Option<Instant>,
    /// Time stood still before.
    stood: Duration,
}

/// The title's time: real time less the time it stood still.
struct Clock {
    state: Mutex<State>,
    freed: Condvar,
}

static TITLE: Clock = Clock::new();

impl Clock {
    const fn new() -> Clock {
        Clock {
            state: Mutex::new(State {
                holds: 0,
                since: None,
                stood: Duration::ZERO,
            }),
            freed: Condvar::new(),
        }
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn hold(&self) {
        let mut s = self.state();
        if s.holds == 0 {
            s.since = Some(Instant::now());
        }
        s.holds += 1;
    }

    fn free(&self) {
        let mut s = self.state();
        s.holds -= 1;
        if s.holds == 0
            && let Some(since) = s.since.take()
        {
            s.stood += since.elapsed();
            self.freed.notify_all();
        }
    }

    fn stood(&self) -> Duration {
        let s = self.state();
        s.stood + s.since.map_or(Duration::ZERO, |since| since.elapsed())
    }

    /// Sleeps `ms` of the title's time with `sleep`, which answers whether
    /// it was cut short (an alertable sleep's call-back): then at once.
    fn sleep(&self, ms: u32, mut sleep: impl FnMut(u32) -> bool) {
        let (began, before) = (Instant::now(), self.stood());
        if sleep(ms) || ms == 0 {
            return;
        }
        let want = Duration::from_millis(ms.into());
        loop {
            let stood = {
                let mut s = self.state();
                while s.holds > 0 {
                    s = self.freed.wait(s).unwrap_or_else(|e| e.into_inner());
                }
                s.stood
            };
            let held = stood.saturating_sub(before);
            if held.is_zero() {
                return; // nothing stood still: the one sleep asked for
            }
            let passed = began.elapsed().saturating_sub(held);
            let Some(left) = want.checked_sub(passed).filter(|l| !l.is_zero()) else {
                return;
            };
            if sleep(u32::try_from(left.as_micros().div_ceil(1000)).unwrap_or(u32::MAX)) {
                return;
            }
        }
    }
}

/// While it lives, the title stands still.
pub(super) struct Still;

impl Still {
    pub(super) fn new() -> Still {
        TITLE.hold();
        Still
    }
}

impl Drop for Still {
    fn drop(&mut self) {
        TITLE.free();
    }
}

/// The time the title stood still so far.
pub fn stood() -> Duration {
    TITLE.stood()
}

static SLEEP: AtomicUsize = AtomicUsize::new(0);
static SLEEP_EX: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn sleep(ms: u32) {
    // SAFETY: what the title's `Sleep` was bound to.
    let real: unsafe extern "system" fn(u32) =
        unsafe { std::mem::transmute(SLEEP.load(Ordering::Relaxed)) };
    TITLE.sleep(ms, |ms| {
        // SAFETY: the title's own call, for as long as is left.
        unsafe { real(ms) };
        false
    });
}

unsafe extern "system" fn sleep_ex(ms: u32, alertable: i32) -> u32 {
    // SAFETY: what the title's `SleepEx` was bound to.
    let real: unsafe extern "system" fn(u32, i32) -> u32 =
        unsafe { std::mem::transmute(SLEEP_EX.load(Ordering::Relaxed)) };
    let mut out = 0;
    TITLE.sleep(ms, |ms| {
        // SAFETY: as above. Not 0: `WAIT_IO_COMPLETION`.
        out = unsafe { real(ms, alertable) };
        out != 0
    });
    out
}

/// What the title's import `name`, found at `address`, is bound to: the
/// address itself, or (`Sleep`, `SleepEx`, with `VICTORY_RECORD`) our
/// wrapper of it.
pub fn import(name: &str, address: usize) -> usize {
    let (slot, ours) = match name {
        "Sleep" => (&SLEEP, sleep as *const () as usize),
        "SleepEx" => (&SLEEP_EX, sleep_ex as *const () as usize),
        _ => return address,
    };
    if address == 0 || super::record::Settings::from_env().is_none() {
        return address;
    }
    // Imported from more than one DLL: each is the system's same function.
    let _ = slot.compare_exchange(0, address, Ordering::Relaxed, Ordering::Relaxed);
    ours
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(ms: u32) -> bool {
        std::thread::sleep(Duration::from_millis(ms.into()));
        false
    }

    #[test]
    fn a_sleep_does_not_count_the_time_stood_still() {
        static CLOCK: Clock = Clock::new();
        let sleeper = std::thread::spawn(|| {
            let began = Instant::now();
            CLOCK.sleep(300, plain);
            began.elapsed()
        });
        std::thread::sleep(Duration::from_millis(50));
        CLOCK.hold();
        std::thread::sleep(Duration::from_millis(400));
        CLOCK.free();
        let took = sleeper.join().unwrap();
        assert!(CLOCK.stood() >= Duration::from_millis(400));
        // 300 ms of the title's time and the 400 stood still.
        assert!(took >= Duration::from_millis(690), "{took:?}");
        assert!(took < Duration::from_millis(1200), "{took:?}");
    }

    #[test]
    fn a_sleep_with_nothing_stood_still_is_one_sleep() {
        let clock = Clock::new();
        let mut calls = Vec::new();
        let began = Instant::now();
        clock.sleep(30, |ms| {
            calls.push(ms);
            plain(ms)
        });
        assert_eq!(calls, [30]);
        assert!(began.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn a_sleep_cut_short_ends_at_once() {
        static CLOCK: Clock = Clock::new();
        CLOCK.hold();
        let sleeper = std::thread::spawn(|| {
            let began = Instant::now();
            let mut calls = 0;
            CLOCK.sleep(10, |_| {
                calls += 1;
                true
            });
            (calls, began.elapsed())
        });
        std::thread::sleep(Duration::from_millis(300));
        CLOCK.free();
        let (calls, took) = sleeper.join().unwrap();
        assert_eq!(calls, 1);
        assert!(took < Duration::from_millis(200), "{took:?}");
    }

    #[test]
    fn nothing_is_wrapped_without_the_recorder() {
        if std::env::var_os("VICTORY_RECORD").is_some() {
            return;
        }
        assert_eq!(import("Sleep", 0x1234), 0x1234);
        assert_eq!(import("SleepEx", 0x1234), 0x1234);
        assert_eq!(import("WaitForSingleObject", 0x1234), 0x1234);
    }
}
