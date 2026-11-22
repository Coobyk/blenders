//! Port of `intern/wm_tooltip.cc` — `wmTooltipState`
//! timer/init/clear/refresh flow + `g_tooltip_time_closed`.

/// `wmTooltipState` verbatim fields.
pub struct WmTooltipState {
    /// `area_from`/`region_from`/`region`.
    pub area_from: usize,
    pub region_from: usize,
    pub region: usize,
    /// `init` callback.
    pub init: usize,
    /// `timer`.
    pub timer: usize,
    /// `pass`/`pass_delay`.
    pub pass: i32,
    /// `event_xy`.
    pub event_xy: (i32, i32),
}

/// `WM_tooltip_time_closed` accessor target.
pub struct TooltipGlobals {
    /// `g_tooltip_time_closed`.
    pub time_closed: f64,
}

/// `WM_tooltip_immediate_init` (verbatim — clear timer,
/// ensure state, init now).
pub fn wm_tooltip_immediate_init(
    state: &mut Option<WmTooltipState>,
    area: usize,
    region: usize,
    init: usize,
    timer_clear: &mut dyn FnMut(&mut WmTooltipState),
    tooltip_init: &mut dyn FnMut(&mut WmTooltipState),
) {
    if state.is_none() {
        *state = Some(WmTooltipState {
            area_from: 0,
            region_from: 0,
            region: 0,
            init: 0,
            timer: 0,
            pass: 0,
            event_xy: (0, 0),
        });
    }
    let s = state.as_mut().unwrap();
    (timer_clear)(s);
    s.area_from = area;
    s.region_from = region;
    s.init = init;
    (tooltip_init)(s);
}

/// `WM_tooltip_timer_init_ex` (verbatim — clear, ensure
/// state, add TIMER delay, save event xy).
pub fn wm_tooltip_timer_init_ex(
    state: &mut Option<WmTooltipState>,
    area: usize,
    region: usize,
    init: usize,
    delay: f64,
    eventstate_xy: (i32, i32),
    timer_clear: &mut dyn FnMut(&mut WmTooltipState),
    event_timer_add: &mut dyn FnMut(f64) -> usize,
) {
    if state.is_none() {
        *state = Some(WmTooltipState {
            area_from: 0,
            region_from: 0,
            region: 0,
            init: 0,
            timer: 0,
            pass: 0,
            event_xy: (0, 0),
        });
    }
    let s = state.as_mut().unwrap();
    (timer_clear)(s);
    s.area_from = area;
    s.region_from = region;
    s.timer = (event_timer_add)(delay);
    s.init = init;
    s.event_xy = eventstate_xy;
}

/// `WM_tooltip_timer_clear` (verbatim — remove timer).
pub fn wm_tooltip_timer_clear(
    state: &mut Option<WmTooltipState>,
    event_timer_remove: &mut dyn FnMut(usize),
) {
    if let Some(s) = state {
        if s.timer != 0 {
            (event_timer_remove)(s.timer);
            s.timer = 0;
        }
    }
}

/// `WM_tooltip_clear` (verbatim — timer + region free +
/// delete, stamp closed time).
pub fn wm_tooltip_clear(
    state: &mut Option<WmTooltipState>,
    globals: &mut TooltipGlobals,
    event_timer_remove: &mut dyn FnMut(usize),
    tooltip_free: &mut dyn FnMut(usize),
    now_seconds: &dyn Fn() -> f64,
) {
    wm_tooltip_timer_clear(state, event_timer_remove);
    if let Some(s) = state {
        if s.region != 0 {
            (tooltip_free)(s.region);
            s.region = 0;
            globals.time_closed = (now_seconds)();
        }
        *state = None;
    }
}
