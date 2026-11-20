//! Port of `BLI_timer.cc` — global timed-callback registry.

use std::sync::Mutex;
use std::time::Instant;

/// `BLI_timer_func` — returns seconds until next call, negative to remove.
pub type TimerFunc = Box<dyn FnMut(u64) -> f64 + Send>;

/// `TimedFunction`.
struct TimedFunction {
    func: TimerFunc,
    next_time: Instant,
    tag_removal: bool,
    persistent: bool,
    uuid: u64,
}

/// `TimerContainer`.
struct TimerContainer {
    funcs: Vec<TimedFunction>,
}

static GLOBAL_TIMER: Mutex<Option<TimerContainer>> = Mutex::new(None);

/// `BLI_timer_register`.
pub fn timer_register(uuid: u64, func: TimerFunc, first_interval: f64, persistent: bool) {
    let mut g = GLOBAL_TIMER.lock().unwrap();
    let c = g.get_or_insert_with(|| TimerContainer { funcs: Vec::new() });
    c.funcs.push(TimedFunction {
        func,
        next_time: Instant::now() + std::time::Duration::from_secs_f64(first_interval.max(0.0)),
        tag_removal: false,
        persistent,
        uuid,
    });
}

/// `BLI_timer_unregister`.
pub fn timer_unregister(uuid: u64) -> bool {
    let mut g = GLOBAL_TIMER.lock().unwrap();
    let Some(c) = g.as_mut() else { return false };
    for f in c.funcs.iter_mut() {
        if f.uuid == uuid && !f.tag_removal {
            f.tag_removal = true;
            return true;
        }
    }
    false
}

/// `BLI_timer_is_registered`.
pub fn timer_is_registered(uuid: u64) -> bool {
    let g = GLOBAL_TIMER.lock().unwrap();
    let Some(c) = g.as_ref() else { return false };
    c.funcs.iter().any(|f| f.uuid == uuid && !f.tag_removal)
}

/// `BLI_timer_execute` — run due callbacks, reap tagged entries.
pub fn timer_execute() {
    let now = Instant::now();
    let mut g = GLOBAL_TIMER.lock().unwrap();
    let Some(c) = g.as_mut() else { return };
    for f in c.funcs.iter_mut() {
        if f.tag_removal || f.next_time > now {
            continue;
        }
        let ret = (f.func)(f.uuid);
        if ret < 0.0 {
            f.tag_removal = true;
        } else {
            f.next_time = now + std::time::Duration::from_secs_f64(ret);
        }
    }
    c.funcs.retain(|f| !f.tag_removal);
}

/// `BLI_timer_free` — remove all.
pub fn timer_free() {
    let mut g = GLOBAL_TIMER.lock().unwrap();
    if let Some(c) = g.as_mut() {
        c.funcs.clear();
    }
}

/// `BLI_timer_on_file_load` — drop non-persistent timers.
pub fn timer_on_file_load() {
    let mut g = GLOBAL_TIMER.lock().unwrap();
    if let Some(c) = g.as_mut() {
        c.funcs.retain(|f| f.persistent);
    }
}
