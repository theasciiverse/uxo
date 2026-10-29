//! Controllers. Gamepads are first-class: every pad is its own player, joins with
//! any button, survives unplugging (its seat waits for it), reads the analog stick
//! and the d-pad, and rumbles when bombs go off near you or you get caught.
//!
//! Keyboard: two schemes on one keyboard, WASD + Space (action F, ride E) and arrows +
//! Enter (action /, ride .). With the kitty keyboard protocol presses and releases are real,
//! so both can play at once; otherwise a key counts as held while its OS repeats keep
//! arriving (Entity's and Marble Recovery's timing tier), and only one key at a time
//! really works.
//!
//! gilrs counts any device with buttons and two axes as a gamepad (drawing-tablet
//! key pads and 3D mice included), so only devices that matched SDL's controller
//! database are used; `UXO_ANY_PAD=1` accepts everything.

use crate::game::{Cmd, Dir};
use log::{debug, info};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};

const PRE_REPEAT_CAP: Duration = Duration::from_millis(220);
const REPEAT_DELAY_DEFAULT: Duration = Duration::from_millis(500);
const REPEAT_INTERVAL_DEFAULT: Duration = Duration::from_millis(33);
const RELEASE_MIN: Duration = Duration::from_millis(60);
const RELEASE_MAX: Duration = Duration::from_millis(160);
const STICK_DEADZONE: f32 = 0.3;
/// How far the other stick axis must lean to count as the turn you want next.
const STICK_TURN: f32 = 0.35;

#[derive(Clone, Copy, Debug)]
struct Repeat {
    delay: Duration,
    interval: Duration,
}

#[derive(Clone, Copy, Default, Debug)]
struct Hold {
    last: Option<Instant>,
    events: u32,
}

impl Hold {
    fn event(&mut self, now: Instant, rep: &mut Repeat) {
        if let Some(last) = self.last {
            let gap = now.duration_since(last);
            let continues = if self.events == 1 {
                gap >= rep.delay.mul_f32(0.5) && gap <= rep.delay.mul_f32(1.6)
            } else {
                gap <= RELEASE_MAX.max(rep.interval * 4)
            };
            if continues {
                self.events += 1;
                let blend = |old: Duration, new: Duration| (old.mul_f32(0.5) + new.mul_f32(0.5)).max(Duration::from_millis(10));
                if self.events == 2 {
                    rep.delay = blend(rep.delay, gap.clamp(Duration::from_millis(100), Duration::from_millis(1000)));
                } else if gap < Duration::from_millis(200) {
                    rep.interval = blend(rep.interval, gap);
                }
                self.last = Some(now);
                return;
            }
        }
        self.last = Some(now);
        self.events = 1;
    }

    fn held(&self, now: Instant, rep: &Repeat) -> bool {
        let Some(last) = self.last else { return false };
        let timeout = if self.events >= 2 {
            (rep.interval * 5 / 2).clamp(RELEASE_MIN, RELEASE_MAX)
        } else {
            PRE_REPEAT_CAP.min(rep.delay + rep.interval * 2)
        };
        now.duration_since(last) <= timeout
    }
}

/// What drives a player.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ctl {
    /// Keyboard scheme 0 (WASD + Space) or 1 (arrows + Enter).
    Keys(usize),
    /// A gamepad, by gilrs id.
    Pad(usize),
}

const DIRS: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

#[derive(Default)]
struct Scheme {
    down: [bool; 4],
    holds: [Hold; 4],
    /// When each direction was last pressed, to know which one is meant.
    pressed_at: [u64; 4],
    bomb: bool,
    action: bool,
    ride: bool,
}

struct Pad {
    id: usize,
    #[cfg(feature = "gamepad")]
    gid: gilrs::GamepadId,
    name: String,
    connected: bool,
    stick: (f32, f32),
    dpad: [bool; 4],
    dpad_at: [u64; 4],
    bomb: bool,
    action: bool,
    ride: bool,
}

pub struct Input {
    exact: bool,
    rep: Repeat,
    keys: [Scheme; 2],
    pads: Vec<Pad>,
    clock: u64,
    /// Fresh direction presses, for menus.
    taps: Vec<(Ctl, Dir)>,
    /// Controls that were used this frame (any key of a scheme, any pad button):
    /// unseated ones join.
    pub touched: Vec<Ctl>,
    pub quit: bool,
    /// Esc, or a pad's Select/Back: out of a mode to the title (and quit from there).
    pub back: bool,
    pub pause: bool,
    pub zoom: i32,
    /// `0`: back to automatic zoom.
    pub zoom_auto: bool,
    #[cfg(feature = "gamepad")]
    gilrs: Option<gilrs::Gilrs>,
    #[cfg(feature = "gamepad")]
    rumbles: Vec<(gilrs::ff::Effect, Instant)>,
}

fn scheme_dir(code: KeyCode) -> Option<(usize, usize)> {
    match code {
        KeyCode::Char('w') | KeyCode::Char('W') => Some((0, 0)),
        KeyCode::Char('s') | KeyCode::Char('S') => Some((0, 1)),
        KeyCode::Char('a') | KeyCode::Char('A') => Some((0, 2)),
        KeyCode::Char('d') | KeyCode::Char('D') => Some((0, 3)),
        KeyCode::Up => Some((1, 0)),
        KeyCode::Down => Some((1, 1)),
        KeyCode::Left => Some((1, 2)),
        KeyCode::Right => Some((1, 3)),
        _ => None,
    }
}

/// `UXO_NO_PADS=1` leaves every pad out (recordings driven from the keyboard);
/// `UXO_ANY_PAD=1` lets in devices without an SDL mapping.
#[cfg(feature = "gamepad")]
fn usable_pad(p: &gilrs::Gamepad) -> bool {
    if std::env::var("UXO_NO_PADS").is_ok_and(|v| v == "1") {
        return false;
    }
    p.mapping_source() == gilrs::MappingSource::SdlMappings || std::env::var("UXO_ANY_PAD").is_ok_and(|v| v == "1")
}

/// Pick the meant direction and the fallback turn from four held flags and when
/// each was pressed: the latest press wins, the latest of the other axis is the turn.
fn choose(held: [bool; 4], at: [u64; 4]) -> (Option<Dir>, Option<Dir>) {
    let latest = |cands: &[usize]| cands.iter().copied().filter(|&d| held[d]).max_by_key(|&d| at[d]);
    let Some(p) = latest(&[0, 1, 2, 3]) else { return (None, None) };
    let other: &[usize] = if p < 2 { &[2, 3] } else { &[0, 1] };
    (Some(DIRS[p]), latest(other).map(|d| DIRS[d]))
}

impl Input {
    pub fn new(exact: bool) -> Input {
        #[cfg(feature = "gamepad")]
        let gilrs = match gilrs::Gilrs::new() {
            Ok(g) => {
                let all: Vec<_> = g.gamepads().map(|(_, p)| (p.name().to_string(), usable_pad(&p), p.is_ff_supported())).collect();
                info!("gamepads at start (name, used, rumble): {all:?}");
                Some(g)
            }
            Err(e) => {
                log::warn!("gamepad support unavailable: {e}");
                None
            }
        };
        let mut input = Input {
            exact,
            rep: Repeat { delay: REPEAT_DELAY_DEFAULT, interval: REPEAT_INTERVAL_DEFAULT },
            keys: Default::default(),
            pads: Vec::new(),
            clock: 0,
            taps: Vec::new(),
            touched: Vec::new(),
            quit: false,
            back: false,
            pause: false,
            zoom: 0,
            zoom_auto: false,
            #[cfg(feature = "gamepad")]
            gilrs,
            #[cfg(feature = "gamepad")]
            rumbles: Vec::new(),
        };
        #[cfg(feature = "gamepad")]
        if let Some(g) = &input.gilrs {
            let found: Vec<Pad> = g.gamepads().filter(|(_, p)| usable_pad(p)).map(|(id, p)| Pad::new(id, p.name())).collect();
            input.pads = found;
        }
        input
    }

    pub fn exact(&self) -> bool {
        self.exact
    }

    pub fn on_key(&mut self, k: KeyEvent, now: Instant) {
        debug!("key {:?} {:?} {:?}", k.code, k.kind, k.modifiers);
        let down = k.kind != KeyEventKind::Release;
        let press = k.kind == KeyEventKind::Press;
        self.clock += 1;
        if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if let Some((s, d)) = scheme_dir(k.code) {
            let sc = &mut self.keys[s];
            if press {
                sc.pressed_at[d] = self.clock;
                self.touched.push(Ctl::Keys(s));
            }
            if self.exact {
                sc.down[d] = down;
                if press {
                    self.taps.push((Ctl::Keys(s), DIRS[d]));
                }
            } else if down {
                if !sc.holds[d].held(now, &self.rep) {
                    sc.pressed_at[d] = self.clock;
                }
                sc.holds[d].event(now, &mut self.rep);
                // legacy terminals send every repeat as a press too; the hold's timing
                // tells a fresh press (a tap) from a repeat
                if sc.holds[d].events == 1 {
                    self.taps.push((Ctl::Keys(s), DIRS[d]));
                }
                // a press of the opposite direction ends that one at once
                sc.holds[d ^ 1] = Hold::default();
            }
            return;
        }
        if !press {
            return;
        }
        match k.code {
            KeyCode::Char(' ') => {
                self.keys[0].bomb = true;
                self.touched.push(Ctl::Keys(0));
            }
            KeyCode::Enter => {
                self.keys[1].bomb = true;
                self.touched.push(Ctl::Keys(1));
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.keys[0].action = true;
                self.touched.push(Ctl::Keys(0));
            }
            KeyCode::Char('/') => {
                self.keys[1].action = true;
                self.touched.push(Ctl::Keys(1));
            }
            // the mount button: E beside WASD, . beside the arrows' /
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.keys[0].ride = true;
                self.touched.push(Ctl::Keys(0));
            }
            KeyCode::Char('.') => {
                self.keys[1].ride = true;
                self.touched.push(Ctl::Keys(1));
            }
            // Q and R sit next to WASD, so leaving is Esc (or Ctrl-C) only
            KeyCode::Esc => self.back = true,
            KeyCode::Char('p') => self.pause = true,
            KeyCode::Char('+') | KeyCode::Char('=') => self.zoom += 1,
            KeyCode::Char('-') => self.zoom -= 1,
            KeyCode::Char('0') => self.zoom_auto = true,
            _ => {}
        }
    }

    /// Drain gamepad events (call once per frame).
    pub fn poll_pads(&mut self) {
        #[cfg(feature = "gamepad")]
        {
            use gilrs::{Axis, Button, EventType};
            let now = Instant::now();
            self.rumbles.retain(|(_, until)| *until > now);
            let Some(g) = &mut self.gilrs else { return };
            while let Some(ev) = g.next_event() {
                let pad = g.gamepad(ev.id);
                if !usable_pad(&pad) {
                    continue;
                }
                let id: usize = ev.id.into();
                let k = match self.pads.iter().position(|p| p.id == id) {
                    Some(k) => k,
                    None => {
                        self.pads.push(Pad::new(ev.id, pad.name()));
                        self.pads.len() - 1
                    }
                };
                self.clock += 1;
                let clock = self.clock;
                let p = &mut self.pads[k];
                match ev.event {
                    EventType::Connected => {
                        p.connected = true;
                        p.name = pad.name().to_string();
                        info!("pad {id} connected: {} (rumble: {})", p.name, pad.is_ff_supported());
                    }
                    EventType::Disconnected => {
                        p.connected = false;
                        p.stick = (0.0, 0.0);
                        p.dpad = [false; 4];
                        info!("pad {id} disconnected: {}", p.name);
                    }
                    EventType::ButtonPressed(b, _) => {
                        debug!("pad {id} press {b:?}");
                        self.touched.push(Ctl::Pad(id));
                        match b {
                            // A and X drop bombs, B is the action button (punch, pick up
                            // and throw, remote), Y rides: your mount's hop or dash
                            Button::South | Button::West => p.bomb = true,
                            Button::East => p.action = true,
                            Button::North => p.ride = true,
                            Button::Start => self.pause = true,
                            Button::Select => self.back = true,
                            Button::DPadUp | Button::DPadDown | Button::DPadLeft | Button::DPadRight => {
                                let d = match b {
                                    Button::DPadUp => 0,
                                    Button::DPadDown => 1,
                                    Button::DPadLeft => 2,
                                    _ => 3,
                                };
                                p.dpad[d] = true;
                                p.dpad_at[d] = clock;
                                self.taps.push((Ctl::Pad(id), DIRS[d]));
                            }
                            _ => {}
                        }
                    }
                    EventType::ButtonReleased(b, _) => {
                        debug!("pad {id} release {b:?}");
                        match b {
                            Button::DPadUp => p.dpad[0] = false,
                            Button::DPadDown => p.dpad[1] = false,
                            Button::DPadLeft => p.dpad[2] = false,
                            Button::DPadRight => p.dpad[3] = false,
                            _ => {}
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickX | Axis::LeftStickY, ..) => {
                        let axis = |a: Axis| pad.axis_data(a).map_or(0.0, |d| d.value());
                        // stick up is positive; screen up is negative
                        let s = (axis(Axis::LeftStickX), -axis(Axis::LeftStickY));
                        let was = p.stick.0.hypot(p.stick.1) >= STICK_DEADZONE;
                        p.stick = s;
                        let is = s.0.hypot(s.1) >= STICK_DEADZONE;
                        if is != was {
                            debug!("pad {id} stick {:.2},{:.2}", s.0, s.1);
                        }
                        // pushing the stick out of its deadzone is a tap that way
                        if is && !was {
                            let d = if s.0.abs() >= s.1.abs() {
                                if s.0 < 0.0 { Dir::Left } else { Dir::Right }
                            } else if s.1 < 0.0 {
                                Dir::Up
                            } else {
                                Dir::Down
                            };
                            self.taps.push((Ctl::Pad(id), d));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// This frame's movement for a controller (the bomb press is taken separately).
    pub fn cmd(&self, ctl: Ctl, now: Instant) -> Cmd {
        let (primary, secondary) = match ctl {
            Ctl::Keys(s) => {
                let sc = &self.keys[s];
                let held: [bool; 4] =
                    std::array::from_fn(|d| if self.exact { sc.down[d] } else { sc.holds[d].held(now, &self.rep) });
                choose(held, sc.pressed_at)
            }
            Ctl::Pad(id) => match self.pads.iter().find(|p| p.id == id) {
                Some(p) if p.dpad.iter().any(|&d| d) => choose(p.dpad, p.dpad_at),
                Some(p) => {
                    let (x, y) = p.stick;
                    if x.hypot(y) < STICK_DEADZONE {
                        (None, None)
                    } else {
                        let h = if x < 0.0 { Dir::Left } else { Dir::Right };
                        let v = if y < 0.0 { Dir::Up } else { Dir::Down };
                        if x.abs() >= y.abs() {
                            (Some(h), (y.abs() >= STICK_TURN).then_some(v))
                        } else {
                            (Some(v), (x.abs() >= STICK_TURN).then_some(h))
                        }
                    }
                }
                None => (None, None),
            },
        };
        Cmd { primary, secondary, bomb: false, action: false, ride: false, tap: None }
    }

    /// Take the oldest pending direction tap.
    pub fn take_tap(&mut self, ctl: Ctl) -> Option<Dir> {
        let k = self.taps.iter().position(|t| t.0 == ctl)?;
        Some(self.taps.remove(k).1)
    }

    /// Take a pending mount press.
    pub fn take_ride(&mut self, ctl: Ctl) -> bool {
        match ctl {
            Ctl::Keys(s) => std::mem::take(&mut self.keys[s].ride),
            Ctl::Pad(id) => self.pads.iter_mut().find(|p| p.id == id).is_some_and(|p| std::mem::take(&mut p.ride)),
        }
    }

    /// Take a pending action press.
    pub fn take_action(&mut self, ctl: Ctl) -> bool {
        match ctl {
            Ctl::Keys(s) => std::mem::take(&mut self.keys[s].action),
            Ctl::Pad(id) => self.pads.iter_mut().find(|p| p.id == id).is_some_and(|p| std::mem::take(&mut p.action)),
        }
    }

    /// Take a pending bomb press (each press drops one bomb).
    pub fn take_bomb(&mut self, ctl: Ctl) -> bool {
        match ctl {
            Ctl::Keys(s) => std::mem::take(&mut self.keys[s].bomb),
            Ctl::Pad(id) => self.pads.iter_mut().find(|p| p.id == id).is_some_and(|p| std::mem::take(&mut p.bomb)),
        }
    }

    pub fn connected(&self, ctl: Ctl) -> bool {
        match ctl {
            Ctl::Keys(_) => true,
            Ctl::Pad(id) => self.pads.iter().any(|p| p.id == id && p.connected),
        }
    }

    /// A short name for the HUD.
    pub fn label(&self, ctl: Ctl) -> String {
        match ctl {
            Ctl::Keys(0) => "WASD".into(),
            Ctl::Keys(_) => "Arrows".into(),
            Ctl::Pad(id) => self.pads.iter().find(|p| p.id == id).map_or("pad".into(), |p| short_name(&p.name)),
        }
    }

    pub fn pad_count(&self) -> usize {
        self.pads.iter().filter(|p| p.connected).count()
    }

    /// Rumble a pad: `strong` and `weak` motor strength 0 to 1, for `ms`.
    #[cfg_attr(not(feature = "gamepad"), allow(unused_variables))]
    pub fn rumble(&mut self, ctl: Ctl, strong: f32, weak: f32, ms: u32) {
        #[cfg(feature = "gamepad")]
        {
            use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
            let Ctl::Pad(id) = ctl else { return };
            let Some(p) = self.pads.iter().find(|p| p.id == id && p.connected) else { return };
            let Some(g) = &mut self.gilrs else { return };
            if !g.gamepad(p.gid).is_ff_supported() {
                return;
            }
            let dur = Ticks::from_ms(ms);
            let mag = |v: f32| (v.clamp(0.0, 1.0) * u16::MAX as f32) as u16;
            let built = EffectBuilder::new()
                .add_effect(BaseEffect {
                    kind: BaseEffectType::Strong { magnitude: mag(strong) },
                    scheduling: Replay { play_for: dur, ..Default::default() },
                    envelope: Default::default(),
                })
                .add_effect(BaseEffect {
                    kind: BaseEffectType::Weak { magnitude: mag(weak) },
                    scheduling: Replay { play_for: dur, ..Default::default() },
                    envelope: Default::default(),
                })
                .repeat(gilrs::ff::Repeat::For(dur))
                .gamepads(&[p.gid])
                .finish(g);
            match built.and_then(|e| e.play().map(|_| e)) {
                // dropping an effect stops it, so it lives until it has played out
                Ok(e) => self.rumbles.push((e, Instant::now() + Duration::from_millis(ms as u64 + 50))),
                Err(e) => debug!("rumble failed on pad {id}: {e}"),
            }
        }
    }
}

impl Pad {
    #[cfg(feature = "gamepad")]
    fn new(gid: gilrs::GamepadId, name: &str) -> Pad {
        Pad {
            id: gid.into(),
            gid,
            name: name.to_string(),
            connected: true,
            stick: (0.0, 0.0),
            dpad: [false; 4],
            dpad_at: [0; 4],
            bomb: false,
            action: false,
            ride: false,
        }
    }
}

/// "Xbox Wireless Controller" -> "Xbox Wireless", keeping HUD labels short.
fn short_name(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().filter(|w| !w.eq_ignore_ascii_case("controller")).collect();
    let s = words.join(" ");
    if s.chars().count() > 16 { s.chars().take(15).collect::<String>() + "." } else if s.is_empty() { "pad".into() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_press_wins_and_other_axis_is_the_turn() {
        // right pressed after down: go right, turn down when blocked
        let (p, s) = choose([false, true, false, true], [0, 5, 0, 9]);
        assert_eq!((p, s), (Some(Dir::Right), Some(Dir::Down)));
        let (p, s) = choose([false, false, false, false], [0; 4]);
        assert_eq!((p, s), (None, None));
    }

    #[test]
    fn short_names() {
        assert_eq!(short_name("Xbox Wireless Controller"), "Xbox Wireless");
        assert_eq!(short_name("Controller"), "pad");
    }
}
