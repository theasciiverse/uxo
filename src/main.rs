//! uXo: bomb-arena battles in the terminal. A tribute to the 90s bomb-arena games.
//!
//! One player practises; a second player joining starts a battle: last one standing
//! takes the round, first to three rounds takes the match, knocked-out players ride
//! the border lobbing bombs in, and after two minutes the arena closes in. Every
//! gamepad is its own player: press any button to join.
//!
//! Run: `cargo run --release` (`RUST_LOG=debug cargo run --release 2>uxo.log` for a
//! session log). Snapshot without a terminal: `cargo run --release -- --snapshot
//! out.ppm [--size 120x42] [--zoom s|4|6|8|12|16|18|24] [--secs t]` renders a staged scene.

mod adventure;
mod baddie;
mod battle;
mod game;
mod input;
mod progress;
mod render;
mod stage;

use adventure::Run;
use battle::{Match, Phase, GO};
use game::{BombKind, Cmd, Event, Game, Item, HZ, MAX_PLAYERS, TILE};
use input::{Ctl, Input};
use log::{debug, info};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event as TermEvent, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::supports_keyboard_enhancement;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget};
use render::{Camera, Zoom, PLAYER};
use std::io;
use std::time::{Duration, Instant};

const FRAME: Duration = Duration::from_millis(16);
const AMBER: Color = Color::Rgb(255, 176, 0);
const DIM: Color = Color::Rgb(120, 120, 130);

fn main() -> io::Result<()> {
    env_logger::Builder::from_default_env().format_timestamp_millis().init();
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = args.iter().position(|a| a == "--from").and_then(|i| args.get(i + 1)) {
        match adventure::index_of(code) {
            Some(i) => {
                let _ = FROM.set(i);
            }
            None => {
                eprintln!("--from takes a stage code like 2-5");
                std::process::exit(2);
            }
        }
    }
    if let Some(i) = args.iter().position(|a| a == "--snapshot") {
        return snapshot(&args, args.get(i + 1).map_or("snapshot.ppm", String::as_str));
    }
    run()
}

/// A fresh seed, or `UXO_SEED` for the same layouts every time (recordings, bug reports).
fn seed() -> u64 {
    if let Some(s) = std::env::var("UXO_SEED").ok().and_then(|v| v.parse().ok()) {
        return s;
    }
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64)
}

/// What's on screen: the title, a battle match, or an adventure run.
enum Mode {
    Title(Title),
    Battle(Match),
    Adventure(Run),
}

impl Mode {
    fn game(&self) -> &Game {
        match self {
            Mode::Title(t) => &t.game,
            Mode::Battle(m) => &m.game,
            Mode::Adventure(r) => &r.game,
        }
    }
    /// Changes when the camera should jump rather than pan (a new round or stage).
    fn scene(&self) -> (u8, u32, usize) {
        match self {
            Mode::Title(_) => (0, 0, 0),
            Mode::Battle(m) => (1, m.round, m.stage_no),
            Mode::Adventure(r) => (2, matches!(r.phase, adventure::Phase::Card(_)) as u32, r.index),
        }
    }
}

/// `--from 2-5`: Continue can start at any stage up to this one, for playtesting.
static FROM: std::sync::OnceLock<usize> = std::sync::OnceLock::new();

/// The furthest stage Continue offers: progress saved, or `--from`.
fn unlocked() -> usize {
    progress::furthest().max(FROM.get().copied().unwrap_or(0)).min(adventure::STAGES.len() - 1)
}

/// The title screen: the logo over a quiet arena with fireworks going up, and a menu.
struct Title {
    game: Game,
    cursor: usize,
    /// The stage Continue starts at; left and right choose any stage reached.
    pick: usize,
}

impl Title {
    fn new(seed: u64) -> Title {
        let mut st = stage::load(0);
        st.items = 0;
        Title { game: Game::from_stage(&st, seed), cursor: 0, pick: unlocked() }
    }
    fn items(&self) -> Vec<(String, TitleChoice)> {
        let mut v = vec![("ADVENTURE".to_string(), TitleChoice::Adventure(0))];
        let far = unlocked();
        if far > 0 {
            let pick = self.pick.min(far);
            // chosen, it shows the arrows that pick the stage in place of the cursor's
            let label = if self.cursor == 1 {
                let (l, r) = (if pick > 0 { "<" } else { " " }, if pick < far { ">" } else { " " });
                format!("CONTINUE {l} {} {r}", adventure::code_of(pick))
            } else {
                format!("CONTINUE {}", adventure::code_of(pick))
            };
            v.push((label, TitleChoice::Adventure(pick)));
        }
        v.push(("BATTLE".to_string(), TitleChoice::Battle));
        v
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TitleChoice {
    Adventure(usize),
    Battle,
}

fn run() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let kitty = supports_keyboard_enhancement().unwrap_or(false);
    if kitty {
        let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES | KeyboardEnhancementFlags::REPORT_EVENT_TYPES;
        execute!(io::stdout(), PushKeyboardEnhancementFlags(flags))?;
    }
    info!("start: kitty keyboard = {kitty}");

    let mut input = Input::new(kitty);
    let mut mode = Mode::Title(Title::new(seed()));
    // seats[i] drives player i
    let mut seats: Vec<Ctl> = Vec::new();
    let mut cam = Camera::new();
    let mut seen_scene = (u8::MAX, 0, 0);
    let mut paused = false;
    let mut shake = 0u32;
    let mut toast: Option<(String, u32)> = None;
    let step = 1.0 / HZ as f32;
    let mut acc = 0.0f32;
    let mut last = Instant::now();
    let mut last_log = Instant::now();
    let mut last_phase = String::new();

    let result = loop {
        let deadline = last + FRAME;
        while event::poll(deadline.saturating_duration_since(Instant::now()))? {
            if let TermEvent::Key(k) = event::read()? {
                input.on_key(k, Instant::now());
            }
        }
        input.poll_pads();
        if input.quit {
            break Ok(());
        }
        if std::mem::take(&mut input.back) {
            if matches!(mode, Mode::Title(_)) {
                break Ok(());
            }
            info!("back to the title");
            mode = Mode::Title(Title::new(seed()));
            paused = false;
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32().min(0.1);
        last = now;

        let mut joined = false;
        for ctl in std::mem::take(&mut input.touched) {
            if !seats.contains(&ctl) && seats.len() < MAX_PLAYERS {
                joined = true;
                seats.push(ctl);
                // a join press is not also a bomb, an action or a pick
                input.take_bomb(ctl);
                input.take_action(ctl);
                input.take_ride(ctl);
                let i = seats.len() - 1;
                match &mut mode {
                    Mode::Battle(m) => {
                        m.add_player();
                    }
                    Mode::Adventure(r) if r.players < 2 => {
                        r.add_player();
                    }
                    _ => {}
                }
                info!("player {} joined with {:?} ({})", i + 1, ctl, input.label(ctl));
                input.rumble(ctl, 0.3, 0.6, 150);
            }
        }
        // joining with Start is not also a pause; the title doesn't pause
        if std::mem::take(&mut input.pause) && !seats.is_empty() && !joined && !matches!(mode, Mode::Title(_)) {
            paused = !paused;
            info!("{}", if paused { "paused" } else { "resumed" });
        }
        let size = terminal.size()?;
        let lay = Lay::of(Rect::new(0, 0, size.width, size.height), if matches!(mode, Mode::Title(_)) { 0 } else { seats.len() });
        if input.zoom != 0 {
            cam.manual = Some(cam.zoom.step(input.zoom));
            input.zoom = 0;
            info!("zoom {}", cam.label());
        }
        if std::mem::take(&mut input.zoom_auto) {
            cam.manual = None;
            info!("zoom auto");
        }

        // paused in the adventure, bomb starts the stage over: the way out of a
        // puzzle gone wrong (a crate in a corner), for a life
        if paused && let Mode::Adventure(r) = &mut mode {
            let mut again = false;
            for &ctl in &seats {
                again |= input.take_bomb(ctl);
            }
            if again && r.start_over() {
                info!("stage started over");
                paused = false;
            }
        }
        acc = if paused { 0.0 } else { acc + dt };
        while acc >= step {
            let mut cmds = [Cmd::default(); MAX_PLAYERS];
            for (i, &ctl) in seats.iter().enumerate() {
                cmds[i] = input.cmd(ctl, Instant::now());
                cmds[i].bomb = input.take_bomb(ctl);
                cmds[i].action = input.take_action(ctl);
                cmds[i].tap = input.take_tap(ctl);
                cmds[i].ride = input.take_ride(ctl);
            }
            let events = match &mut mode {
                Mode::Title(t) => {
                    let pick = title_step(t, &cmds, seats.len());
                    if let Some(choice) = pick {
                        info!("title: {choice:?} with {} players", seats.len());
                        mode = match choice {
                            TitleChoice::Adventure(start) => Mode::Adventure(Run::new(seed(), seats.len().min(2), start)),
                            TitleChoice::Battle => Mode::Battle(Match::new(seed(), seats.len())),
                        };
                    }
                    Vec::new()
                }
                Mode::Battle(m) => m.step(&cmds),
                Mode::Adventure(r) => {
                    let ev = r.step(&cmds);
                    if r.phase == adventure::Phase::Done && cmds.iter().any(|c| c.bomb) {
                        mode = Mode::Title(Title::new(seed()));
                    }
                    ev
                }
            };
            for ev in events {
                feel(ev, mode.game(), &seats, &mut input, &mut shake, &mut toast);
            }
            let phase = match &mode {
                Mode::Title(_) => "title".to_string(),
                Mode::Battle(m) => format!("battle {:?}", std::mem::discriminant(&m.phase)),
                Mode::Adventure(r) => format!("adventure {} {:?}", r.code(), std::mem::discriminant(&r.phase)),
            };
            if phase != last_phase {
                match &mode {
                    Mode::Battle(m) => info!("battle phase {:?} (round {}, wins {:?})", m.phase, m.round, &m.wins[..seats.len().min(MAX_PLAYERS)]),
                    Mode::Adventure(r) => info!("adventure {} {:?} (lives {:?}, score {:?})", r.code(), r.phase, &r.lives[..r.players], &r.score[..r.players]),
                    Mode::Title(_) => info!("title"),
                }
                last_phase = phase;
            }
            acc -= step;
        }
        shake = shake.saturating_sub(1);
        if let Some((_, t)) = &mut toast {
            *t = t.saturating_sub(1);
            if *t == 0 {
                toast = None;
            }
        }
        let scene = mode.scene();
        cam.update(mode.game(), lay.play, dt, scene != seen_scene);
        seen_scene = scene;

        if now.duration_since(last_log) >= Duration::from_millis(500) {
            last_log = now;
            let g = mode.game();
            let players: Vec<String> = seats
                .iter()
                .enumerate()
                .map(|(i, &ctl)| {
                    let p = &g.players[i];
                    let c = input.cmd(ctl, now);
                    format!(
                        "p{} {:?} at {},{} tile {:?} alive {} out {} rider {:?} mount {:?} want {:?}/{:?} b{} f{} s{} {:?} dis {:?}",
                        i + 1, ctl, p.x, p.y, p.tile(), p.alive, p.out, p.rider, p.mount, c.primary, c.secondary, p.bombs, p.fire, p.speed, p.kind, p.disease
                    )
                })
                .collect();
            debug!(
                "snapshot: {last_phase} step {} zoom {} cam {:?} paused {paused} bombs {} baddies {} pads {} | {}",
                g.steps,
                cam.label(),
                cam.at(),
                g.bombs.len(),
                g.baddies.len(),
                input.pad_count(),
                players.join(" | ")
            );
        }

        let shake_off = if shake > 0 { [1i16, -1][(shake % 2) as usize] } else { 0 };
        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();
            match &mode {
                Mode::Title(t) => {
                    let full = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(2));
                    render::draw_arena(&t.game, cam.zoom, cam.at(), 0, full, buf);
                    render::dim(buf, full, 0.4);
                    title_screen(t, &input, &seats, area, buf);
                }
                Mode::Battle(m) => {
                    render::draw_arena(&m.game, cam.zoom, cam.at(), shake_off, lay.play, buf);
                    hud(m, &input, &seats, &cam, &lay, area, buf);
                    if seats.is_empty() {
                        banner(&["Press any button on a gamepad to join", "or WASD + Space, or the arrows + Enter"], area, buf);
                    } else if paused {
                        banner(&["PAUSED", "P or Start to play on", "Esc or Select: back to the title"], area, buf);
                    } else {
                        phase_banner(m, area, buf);
                    }
                }
                Mode::Adventure(r) => {
                    render::draw_arena(&r.game, cam.zoom, cam.at(), shake_off, lay.play, buf);
                    adv_hud(r, &input, &seats, &cam, &lay, area, buf);
                    if paused {
                        banner(&["PAUSED", "P or Start to play on", "bomb: start the stage over (costs a life)", "Esc or Select: back to the title"], area, buf);
                    } else {
                        adv_banner(r, area, buf);
                    }
                }
            }
            if let Some((text, _)) = &toast {
                let w = render::pixel_width(text);
                let x = area.x + area.width.saturating_sub(w) / 2;
                render::pixel_text_big(buf, x, lay.play.y + 2, text, &[[255, 220, 90]], 1, area);
            }
        })?;
    };

    if kitty {
        let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
    }
    ratatui::restore();
    result
}

/// The title: fireworks over a quiet arena, the menu picked with taps and a bomb.
fn title_step(t: &mut Title, cmds: &[Cmd; MAX_PLAYERS], seated: usize) -> Option<TitleChoice> {
    let g = &mut t.game;
    g.step(&[Cmd::default(); MAX_PLAYERS]);
    if g.steps % 55 == 0 {
        let x = (g.rand_pub(g.w as u32 - 2) as i32 + 1) * TILE;
        let y = (g.rand_pub(g.h as u32 / 2) as i32 + 1) * TILE;
        g.fireworks(x, y, 32);
    }
    let n = t.items().len();
    for c in cmds.iter() {
        match c.tap {
            Some(game::Dir::Up) => t.cursor = (t.cursor + n - 1) % n,
            Some(game::Dir::Down) => t.cursor = (t.cursor + 1) % n,
            // on Continue, left and right choose the stage
            Some(game::Dir::Left) if t.cursor == 1 && n == 3 => t.pick = t.pick.min(unlocked()).saturating_sub(1),
            Some(game::Dir::Right) if t.cursor == 1 && n == 3 => t.pick = (t.pick + 1).min(unlocked()),
            _ => {}
        }
    }
    t.cursor = t.cursor.min(n - 1);
    if seated > 0 && cmds.iter().any(|c| c.bomb) {
        return Some(t.items()[t.cursor].1);
    }
    None
}

fn title_screen(t: &Title, input: &Input, seats: &[Ctl], area: Rect, buf: &mut Buffer) {
    let amber = [255, 176, 0];
    let flash = (t.game.steps / 20) % 2 == 0;
    let x_col = if flash { [255, 70, 40] } else { [255, 225, 70] };
    // the logo, big
    let scale = if area.width >= 160 && area.height >= 44 { 6 } else if area.width >= 100 { 4 } else { 2 };
    let w = render::pixel_width("uXo") as i32 * scale;
    let x = area.x + (area.width as i32 - w).max(0) as u16 / 2;
    let y = area.y + area.height / 6;
    render::pixel_text_big(buf, x, y, "uXo", &[amber, x_col, amber], scale, area);
    // the line ends in a cursor, blinking at a terminal's pace: how Dot came in
    let tag = "Blast your way out to the real world!";
    let blink = if (t.game.steps / 30) % 2 == 0 { "_" } else { " " };
    let ink = Style::default().fg(Color::Rgb(230, 230, 236)).bg(Color::Rgb(10, 10, 14));
    let tag = Line::from(vec![Span::raw(" "), Span::styled(tag, ink), Span::styled(blink, ink.fg(Color::Rgb(255, 176, 0)))]);
    Paragraph::new(tag.centered())
        .render(Rect::new(area.x, y + (6 * scale / 2) as u16 + 1, area.width, 1), buf);
    // the menu
    let mut my = y + (6 * scale / 2) as u16 + 4;
    for (n, (label, _)) in t.items().iter().enumerate() {
        let on = n == t.cursor;
        let text = if on && !label.starts_with("CONTINUE") { format!("> {label} <") } else { label.clone() };
        let w = render::pixel_width(&text);
        let col = if on { [255, 220, 90] } else { [200, 200, 210] };
        render::pixel_text_big(buf, area.x + area.width.saturating_sub(w) / 2, my, &text, &[col], 1, area);
        my += 4;
    }
    // who is in
    let mut who = vec![];
    for (i, &ctl) in seats.iter().enumerate() {
        who.push(Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(player_color(i))));
        who.push(Span::styled(format!(" {}   ", input.label(ctl)), Style::default().fg(Color::Rgb(220, 220, 225)).bg(Color::Rgb(10, 10, 14))));
    }
    let hint = if seats.is_empty() {
        "press any button on a gamepad, or Space or Enter, to join".to_string()
    } else {
        "up/down to choose, left/right picks the stage, bomb to start; more players can join any time".to_string()
    };
    who.push(Span::styled(hint, Style::default().fg(DIM).bg(Color::Rgb(10, 10, 14))));
    Paragraph::new(Line::from(who).centered()).render(Rect::new(area.x, my + 1, area.width, 1), buf);
    let foot = "Esc quits · +/- 0 size";
    Paragraph::new(Line::from(foot).centered()).style(Style::default().fg(DIM)).render(Rect::new(area.x, area.bottom().saturating_sub(2), area.width, 1), buf);
}

/// Rumble, shake and the log for one game event.
fn feel(ev: Event, g: &Game, seats: &[Ctl], input: &mut Input, shake: &mut u32, toast: &mut Option<(String, u32)>) {
    let pad = |i: usize| seats.get(i).copied();
    match ev {
        Event::Boom { x, y } => {
            *shake = (*shake).max(8);
            // everyone near the blast feels it, closer is stronger
            for (i, &ctl) in seats.iter().enumerate() {
                let p = &g.players[i];
                if !p.alive {
                    continue;
                }
                let (px, py) = p.tile();
                let dist = px.abs_diff(x) + py.abs_diff(y);
                if dist <= 4 {
                    let s = 1.0 - dist as f32 / 5.0;
                    input.rumble(ctl, 0.5 * s, 0.9 * s, 120);
                }
            }
            debug!("boom at {x},{y}");
        }
        Event::Death { player, by } => {
            info!("player {} knocked out by {:?} (deaths {})", player + 1, by.map(|b| b + 1), g.players[player].deaths);
            if let Some(c) = pad(player) {
                input.rumble(c, 1.0, 1.0, 400);
            }
        }
        Event::Saved { player } => {
            info!("player {}'s heart took the hit", player + 1);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.8, 0.4, 200);
            }
        }
        Event::Pickup { player, item } => {
            info!("player {} picked up {item:?}", player + 1);
            if let Some(c) = pad(player) {
                let strong = if item == Item::Skull { 0.6 } else { 0.0 };
                input.rumble(c, strong, 0.4, 60);
            }
        }
        Event::Kick { player } | Event::Throw { player } => {
            debug!("player {} {:?}", player + 1, ev);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.0, 0.3, 50);
            }
        }
        Event::Stunned { player } => {
            info!("player {} stunned", player + 1);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.7, 0.2, 250);
            }
        }
        Event::Infect { from, to } => {
            info!("player {} passed a disease to player {}", from + 1, to + 1);
            if let Some(c) = pad(to) {
                input.rumble(c, 0.5, 0.0, 150);
            }
        }
        Event::Comeback { player } => {
            info!("player {} is back from the border", player + 1);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.6, 1.0, 300);
            }
        }
        Event::Hurry => {
            info!("hurry up: sudden death");
            for &c in seats {
                input.rumble(c, 0.4, 0.4, 500);
            }
        }
        Event::Pressure { x, y } => {
            for (i, &ctl) in seats.iter().enumerate() {
                let p = &g.players[i];
                let (px, py) = p.tile();
                if p.alive && px.abs_diff(x) + py.abs_diff(y) <= 2 {
                    input.rumble(ctl, 0.3, 0.0, 60);
                }
            }
        }
        Event::Drop { player } => debug!("player {} dropped a bomb", player + 1),
        Event::Warp { player } => {
            debug!("player {} warped", player + 1);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.0, 0.5, 120);
            }
        }
        Event::BaddieDown { species, by, .. } => {
            let sp = &g.species[species];
            info!("{} beaten by player {}", sp.name, by + 1);
            if let Some(c) = pad(by) {
                input.rumble(c, 0.2, 0.5, 80);
            }
            if sp.size > 1 {
                *shake = 30;
                for &c in seats {
                    input.rumble(c, 1.0, 1.0, 700);
                }
            }
        }
        Event::BaddieHit { species } => debug!("{} hit", g.species[species].name),
        Event::DudPopped { x, y, by } => {
            info!("dud at {x},{y} set off by player {}", by + 1);
            for &c in seats {
                input.rumble(c, 0.0, 0.6, 150);
            }
        }
        Event::ExitOpen => {
            info!("the exit is open");
            *toast = Some(("THE HATCH IS OPEN!".to_string(), HZ * 3));
            for &c in seats {
                input.rumble(c, 0.3, 0.3, 200);
            }
        }
        Event::Cleared => info!("stage cleared"),
        Event::Mounted { player } => {
            info!("player {} is riding", player + 1);
            *toast = Some(("TAMED! RIDE WITH Y, E OR .".to_string(), HZ * 3));
            if let Some(c) = pad(player) {
                input.rumble(c, 0.3, 0.8, 200);
            }
        }
        Event::MountLost { player } => {
            info!("player {}'s mount took the hit", player + 1);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.9, 0.3, 250);
            }
        }
        Event::Shoved { player } => {
            debug!("player {} pushed a block", player + 1);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.2, 0.0, 60);
            }
        }
        Event::DoorOpened { player } => {
            info!("player {} unlocked a door", player + 1);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.0, 0.5, 100);
            }
        }
        Event::Gates { open } => info!("gates {}", if open { "open" } else { "shut" }),
        Event::Got { player, what } => {
            info!("player {} picked up {what:?}", player + 1);
            if let Some(c) = pad(player) {
                input.rumble(c, 0.0, 0.4, 60);
            }
        }
        Event::Fizzled { x, y } => info!("a bomb fizzled at {x},{y}: UXO"),
        Event::SocketOpen => {
            info!("every part found: the sockets open");
            *toast = Some(("EVERY PART FOUND!".to_string(), HZ * 3));
        }
    }
}

/// What a player has, compactly: "3b 4f 1s KPGL remote <3" (narrow terminals).
fn kit(p: &game::Player) -> String {
    let mut s = format!("{}b {}f {}s", p.bombs, p.fire, p.speed);
    let flags: String = [(p.kick, 'K'), (p.punch, 'P'), (p.glove, 'G'), (p.line, 'L'), (p.bomb_pass, 'B'), (p.block_pass, 'W')]
        .iter()
        .filter(|f| f.0)
        .map(|f| f.1)
        .collect();
    if !flags.is_empty() {
        s += &format!(" {flags}");
    }
    s += match p.kind {
        BombKind::Normal => "",
        BombKind::Remote => " remote",
        BombKind::Pierce => " pierce",
        BombKind::Rubber => " rubber",
        BombKind::Mine => " mine",
        BombKind::Surge => " surge",
    };
    if p.heart {
        s += " <3";
    }
    if let Some(d) = p.disease {
        s += &format!(" sick:{d:?}");
    }
    s
}

/// The items a player holds that change how they play, as badges.
fn badges(p: &game::Player) -> Vec<Item> {
    let mut v = Vec::new();
    for (has, item) in [(p.kick, Item::Kick), (p.punch, Item::Punch), (p.glove, Item::Glove), (p.line, Item::LineBomb)] {
        if has {
            v.push(item);
        }
    }
    match p.kind {
        BombKind::Remote => v.push(Item::Remote),
        BombKind::Pierce => v.push(Item::Pierce),
        BombKind::Rubber => v.push(Item::Rubber),
        BombKind::Mine => v.push(Item::Mine),
        BombKind::Surge => v.push(Item::Surge),
        BombKind::Normal => {}
    }
    for (has, item) in [(p.bomb_pass, Item::BombPass), (p.block_pass, Item::BlockPass), (p.heart, Item::Heart), (p.vest > 0, Item::Vest), (p.disease.is_some(), Item::Skull)] {
        if has {
            v.push(item);
        }
    }
    v
}

fn player_color(i: usize) -> Color {
    let [r, g, b] = PLAYER[i];
    Color::Rgb(r, g, b)
}

fn stars(m: &Match, i: usize) -> String {
    (0..m.options.to_win).map(|k| if k < m.wins[i] { '*' } else { '.' }).collect()
}

/// Where we are: the stage, the round, the clock.
fn status_line(m: &Match) -> Vec<Span<'static>> {
    let g = &m.game;
    let white = Style::default().fg(Color::Rgb(225, 225, 230));
    let dim = Style::default().fg(DIM);
    let stage = format!("STAGE {}/{} ", m.stage_no + 1, stage::count());
    let mut v = vec![Span::styled(stage, dim), Span::styled(m.stage.name.clone(), white.add_modifier(Modifier::BOLD))];
    match m.phase {
        Phase::Practice => v.push(Span::styled("   PRACTICE", dim)),
        Phase::Lobby(_) => v.push(Span::styled("   BATTLE SETUP", dim)),
        _ => {
            v.push(Span::styled(format!("   ROUND {} ", m.round), dim));
            v.push(Span::styled(format!("first to {}", m.options.to_win), dim));
            let flash = (g.steps / 20) % 2 == 0;
            if g.hurry() {
                v.push(Span::raw("   "));
                v.push(Span::styled(" HURRY! ", Style::default().fg(Color::Black).bg(if flash { Color::Rgb(255, 70, 40) } else { AMBER }).add_modifier(Modifier::BOLD)));
            } else if g.hurry_at != u32::MAX {
                let left = g.hurry_at.saturating_sub(g.clock) / HZ;
                v.push(Span::styled(format!("   {}:{:02}", left / 60, left % 60), white.add_modifier(Modifier::BOLD)));
            }
        }
    }
    v
}

/// How the screen is split: the top bar, an optional player row, player cards at the
/// sides when there is room, the play view, and the hint row one up from the bottom.
struct Lay {
    /// Rows the top bar takes: 3 in the pixel font, 1 as plain text when tight.
    bar: u16,
    /// The compact player row under the bar (when there are no cards).
    players_row: Option<u16>,
    cards: bool,
    /// Width of each side column for the cards.
    side: u16,
    play: Rect,
}

impl Lay {
    fn of(area: Rect, seated: usize) -> Lay {
        let bar = if area.width >= 80 && area.height >= 20 { 3 } else { 1 };
        let cards = seated > 0 && area.width >= 190 && area.height >= 34;
        let side = if cards { 30 } else { 0 };
        let players_row = (!cards && seated > 0).then_some(area.y + bar);
        let top = area.y + bar + players_row.map_or(0, |_| 1);
        // the hint row is one up from the bottom, with the last row left clear
        let bottom = area.bottom().saturating_sub(2);
        let play = Rect::new(area.x + side, top, area.width.saturating_sub(side * 2), bottom.saturating_sub(top));
        Lay { bar, players_row, cards, side, play }
    }
}

const BAR_BG: render::Rgb = [16, 16, 22];

/// The top bar in the pixel font: the logo at the left, a place in the middle (a small
/// dim prefix and a name), a label at the right.
fn draw_bar(buf: &mut Buffer, area: Rect, steps: u64, small: &str, name: &str, right: &str, right_col: render::Rgb) {
    let r = Rect::new(area.x, area.y, area.width, 3);
    render::fill(buf, r, BAR_BG);
    let amber = [255, 176, 0];
    let dim = [120, 120, 132];
    let white = [230, 230, 236];
    let flash = (steps / 20) % 2 == 0;
    // the logo, the X flashing like a fuse
    let mut x = area.x + 2;
    x += render::pixel_text(buf, x, area.y, "u", amber, BAR_BG, area);
    x += render::pixel_text(buf, x, area.y, "X", if flash { [255, 70, 40] } else { [255, 225, 70] }, BAR_BG, area);
    render::pixel_text(buf, x, area.y, "o", amber, BAR_BG, area);

    let right_x = area.right().saturating_sub(render::pixel_width(right) + 2);
    render::pixel_text(buf, right_x, area.y, right, right_col, BAR_BG, area);

    // the place, centred in the room between the logo and the label; the prefix goes
    // first, then the name is shortened, when space is tight
    let (lo, hi) = (area.x + 16, right_x.saturating_sub(4));
    let room = hi.saturating_sub(lo);
    let mut name = name.to_uppercase();
    let with_small = render::pixel_width(small) + render::pixel_width(&name) <= room;
    while render::pixel_width(&name) > room && !name.is_empty() {
        name.pop();
    }
    let w = render::pixel_width(&name) + if with_small { render::pixel_width(small) } else { 0 };
    let mut x = lo + room.saturating_sub(w) / 2;
    if with_small {
        x += render::pixel_text(buf, x, area.y, small, dim, BAR_BG, area);
    }
    render::pixel_text(buf, x, area.y, name.trim_end(), white, BAR_BG, area);
}

/// The battle's bar: the stage, the round and the clock.
fn top_bar(m: &Match, area: Rect, buf: &mut Buffer) {
    let g = &m.game;
    let amber = [255, 176, 0];
    let dim = [120, 120, 132];
    let white = [230, 230, 236];
    let flash = (g.steps / 20) % 2 == 0;
    let (label, col) = match m.phase {
        Phase::Practice => ("PRACTICE".to_string(), dim),
        Phase::Lobby(_) => ("SETUP".to_string(), dim),
        _ if g.hurry() => ("HURRY!".to_string(), if flash { [255, 70, 40] } else { amber }),
        _ if g.hurry_at == u32::MAX => (format!("ROUND {}", m.round), white),
        _ => {
            let left = g.hurry_at.saturating_sub(g.clock) / HZ;
            (format!("ROUND {}  {}:{:02}", m.round, left / 60, left % 60), white)
        }
    };
    let small = format!("{}/{} ", m.stage_no + 1, stage::count());
    draw_bar(buf, area, g.steps, &small, &m.stage.name, &label, col);
}

/// The top bar, player cards (or the compact player row) and the hint row.
fn hud(m: &Match, input: &Input, seats: &[Ctl], cam: &Camera, lay: &Lay, area: Rect, buf: &mut Buffer) {
    let g = &m.game;
    if lay.bar == 3 {
        top_bar(m, area, buf);
    } else {
        let mut top = vec![Span::styled(" uXo   ", Style::default().fg(AMBER).add_modifier(Modifier::BOLD))];
        top.extend(status_line(m));
        Paragraph::new(Line::from(top)).render(Rect::new(area.x, area.y, area.width, 1), buf);
    }

    if let Some(row) = lay.players_row {
        let mut spans = Vec::new();
        for (i, &ctl) in seats.iter().enumerate() {
            let p = &g.players[i];
            spans.push(Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(player_color(i))));
            if m.battle() {
                spans.push(Span::styled(format!(" {}", stars(m, i)), Style::default().fg(AMBER)));
            }
            spans.push(Span::styled(format!(" {}   ", player_state(m, input, i, ctl).unwrap_or_else(|| kit(p))), Style::default().fg(Color::Rgb(220, 220, 225))));
        }
        Paragraph::new(Line::from(spans).centered()).render(Rect::new(area.x, row, area.width, 1), buf);
    }

    if lay.cards {
        let body = Rect::new(area.x, lay.play.y, area.width, lay.play.height);
        let card_w = lay.side - 4;
        let card_h = (body.height / 2).min(17);
        let left = area.x + 2;
        let right = area.right() - lay.side + 2;
        // P1 top-left, P2 bottom-right, P3 top-right, P4 bottom-left, like the starts
        let spots = [(left, body.y), (right, body.bottom() - card_h), (right, body.y), (left, body.bottom() - card_h)];
        for (i, &ctl) in seats.iter().enumerate() {
            let (x, y) = spots[i];
            card(m, input, i, ctl, Rect::new(x, y, card_w, card_h), area, buf);
        }
    }

    let keys = if input.exact() { "keys: exact" } else { "keys: one at a time" };
    let hint = format!(
        "bomb: A X Space Enter · action: B Y F / · pause: Start P · size: +/- 0 ({}) · quit: Esc · pads: {} · {keys}",
        cam.label(),
        input.pad_count()
    );
    if area.height >= 3 {
        Paragraph::new(Line::from(hint).centered())
            .style(Style::default().fg(DIM))
            .render(Rect::new(area.x, area.bottom() - 2, area.width, 1), buf);
    }
}

/// A short state instead of the kit, when there is one.
fn player_state(m: &Match, input: &Input, i: usize, ctl: Ctl) -> Option<String> {
    let p = &m.game.players[i];
    if !input.connected(ctl) {
        Some("pad unplugged".into())
    } else if m.waiting(i) {
        Some("joins next round".into())
    } else if p.rider.is_some() {
        Some("riding".into())
    } else if p.out {
        Some("out".into())
    } else {
        None
    }
}

/// One player's card: hero, wins, match numbers, current stats and item badges.
fn card(m: &Match, input: &Input, i: usize, ctl: Ctl, r: Rect, area: Rect, buf: &mut Buffer) {
    let p = &m.game.players[i];
    let col = player_color(i);
    let title = Line::from(vec![
        Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(col).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {} ", input.label(ctl)), Style::default().fg(col)),
    ]);
    let block = Block::bordered().border_type(BorderType::Rounded).border_style(Style::default().fg(col)).title(title);
    let inner = block.inner(r);
    Clear.render(r, buf);
    block.render(r, buf);
    if inner.height < 5 {
        return;
    }
    render::hero_badge(buf, inner.x + 1, inner.y, i, p, m.game.steps, area);
    let white = Style::default().fg(Color::Rgb(225, 225, 230));
    let dim = Style::default().fg(DIM);
    let t = &m.tally[i];
    let mut right: Vec<Line> = Vec::new();
    if m.battle() {
        right.push(Line::from(Span::styled(stars(m, i), Style::default().fg(AMBER).add_modifier(Modifier::BOLD))));
        right.push(Line::from(vec![Span::styled("KOs ", dim), Span::styled(format!("{}", t.kills + p.kills), white)]));
        right.push(Line::from(vec![Span::styled("blocks ", dim), Span::styled(format!("{}", t.blocks + p.blocks), white)]));
    } else {
        right.push(Line::from(Span::styled("practice", dim)));
        right.push(Line::from(vec![Span::styled("blocks ", dim), Span::styled(format!("{}", p.blocks), white)]));
    }
    if let Some(s) = player_state(m, input, i, ctl) {
        right.push(Line::from(Span::styled(s, Style::default().fg(AMBER))));
    } else if let Some(d) = p.disease {
        right.push(Line::from(Span::styled(format!("sick: {d:?}"), Style::default().fg(Color::Rgb(190, 120, 230)))));
    }
    Paragraph::new(right).render(Rect::new(inner.x + 11, inner.y, inner.width.saturating_sub(11), 4), buf);
    if inner.height < 6 {
        return;
    }
    let stats = Line::from(vec![
        Span::styled("bombs ", dim),
        Span::styled(format!("{}", p.bombs), white),
        Span::styled(" fire ", dim),
        Span::styled(format!("{}", p.fire), white),
        Span::styled(" speed ", dim),
        Span::styled(format!("{}", p.speed), white),
    ]);
    Paragraph::new(stats).render(Rect::new(inner.x + 1, inner.y + 5, inner.width.saturating_sub(1), 1), buf);
    // the special items, three badges to a row
    let room = inner.height.saturating_sub(7) / 3;
    for (n, item) in badges(p).into_iter().enumerate().take(room as usize * 3) {
        let (bx, by) = (inner.x + 1 + (n as u16 % 3) * 7, inner.y + 7 + (n as u16 / 3) * 3);
        render::item_badge(buf, bx, by, item, area);
    }
}

fn phase_banner(m: &Match, area: Rect, buf: &mut Buffer) {
    let name = |i: usize| format!("P{}", i + 1);
    let gold = Style::default().fg(AMBER).add_modifier(Modifier::BOLD);
    let white = Style::default().fg(Color::Rgb(225, 225, 230));
    let dim = Style::default().fg(DIM);
    match m.phase {
        Phase::Lobby(row) => {
            let mut lines = vec![Line::from(Span::styled("BATTLE SETUP", gold)).centered(), Line::raw("")];
            let mut who = vec![Span::styled("Players  ", dim)];
            for i in 0..m.players {
                who.push(Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(player_color(i))));
                who.push(Span::raw(" "));
            }
            if m.players < MAX_PLAYERS {
                who.push(Span::styled(" any button joins", dim));
            }
            lines.push(Line::from(who));
            lines.push(Line::raw(""));
            for r in 0..battle::ROWS {
                let (label, value) = m.options.row(r);
                let on = r == row;
                let mark = if on { "> " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(mark, gold),
                    Span::styled(format!("{label:<24}"), if on { white.add_modifier(Modifier::BOLD) } else { dim }),
                    Span::styled(if on { format!("< {value} >") } else { format!("  {value}  ") }, if on { gold } else { white }),
                ]));
            }
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(m.stage.about.clone(), dim)).centered());
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled("up/down choose · left/right change · bomb starts", dim)).centered());
            panel(lines, area, buf);
        }
        Phase::Ready(_) => banner(&[&format!("ROUND {}", m.round), &m.stage.name, "READY?"], area, buf),
        Phase::Fight if m.game.clock < GO => banner(&["GO!"], area, buf),
        Phase::RoundOver { winner, .. } => {
            let head = match winner {
                Some(w) => format!("{} TAKES THE ROUND", name(w)),
                None => "DRAW".to_string(),
            };
            let mut lines = vec![Line::from(Span::styled(head, gold)).centered(), Line::raw("")];
            lines.extend(table(m));
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(format!("next: stage {}", next_stage_name(m)), dim)).centered());
            panel(lines, area, buf);
        }
        Phase::MatchOver { winner } => {
            let mut lines = vec![Line::from(Span::styled(format!("{} WINS THE MATCH!", name(winner)), gold)).centered(), Line::raw("")];
            lines.extend(table(m));
            let awards = m.awards();
            if !awards.is_empty() {
                lines.push(Line::raw(""));
                for (title, i) in awards {
                    lines.push(Line::from(vec![
                        Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(player_color(i))),
                        Span::styled(format!("  {title}"), white),
                    ]));
                }
            }
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled("bomb for the setup screen", dim)).centered());
            panel(lines, area, buf);
        }
        Phase::Practice if m.players == 1 && m.game.steps < 300 => banner(&["Practice", "a second player starts a battle"], area, buf),
        _ => {}
    }
}

fn next_stage_name(m: &Match) -> String {
    let n = m.options.stage.unwrap_or((m.stage_no + 1) % stage::count());
    stage::load(n).name
}

/// The match so far, one row per player.
fn table(m: &Match) -> Vec<Line<'static>> {
    let dim = Style::default().fg(DIM);
    let white = Style::default().fg(Color::Rgb(225, 225, 230));
    let mut out = vec![Line::from(Span::styled(format!("{:<6}{:<8}{:>5}{:>6}{:>8}{:>7}{:>7}", "", "wins", "KOs", "outs", "blocks", "items", "bombs"), dim))];
    for i in 0..m.players {
        let t = &m.tally[i];
        out.push(Line::from(vec![
            Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(player_color(i))),
            Span::raw("  "),
            Span::styled(format!("{:<8}", stars(m, i)), Style::default().fg(AMBER)),
            Span::styled(format!("{:>5}{:>6}{:>8}{:>7}{:>7}", t.kills, t.outs, t.blocks, t.items, t.bombs), white),
        ]));
    }
    out
}

/// A boxed panel in the middle of the screen.
fn panel(lines: Vec<Line>, area: Rect, buf: &mut Buffer) {
    let w = lines.iter().map(|l| l.width()).max().unwrap_or(0) as u16 + 6;
    let h = lines.len() as u16 + 2;
    let r = Rect::new(area.x + area.width.saturating_sub(w) / 2, area.y + area.height.saturating_sub(h) / 2, w.min(area.width), h.min(area.height));
    Clear.render(r, buf);
    let block = Block::bordered().border_type(BorderType::Rounded).border_style(Style::default().fg(AMBER)).style(Style::default().bg(Color::Rgb(8, 8, 12)));
    let inner = block.inner(r);
    block.render(r, buf);
    let inner = Rect::new(inner.x + 2, inner.y, inner.width.saturating_sub(4), inner.height);
    Paragraph::new(lines).render(inner, buf);
}

fn banner(lines: &[&str], area: Rect, buf: &mut Buffer) {
    let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16 + 6;
    let h = lines.len() as u16 + 2;
    let r = Rect::new(area.x + area.width.saturating_sub(w) / 2, area.y + area.height.saturating_sub(h) / 2, w.min(area.width), h.min(area.height));
    Clear.render(r, buf);
    let text: Vec<Line> = std::iter::once(Line::raw("")).chain(lines.iter().map(|l| Line::raw(*l).centered())).collect();
    Paragraph::new(text)
        .style(Style::default().bg(Color::Rgb(0, 0, 0)).fg(AMBER).add_modifier(Modifier::BOLD))
        .render(r, buf);
}

/// The adventure's bar, player cards or row, and hints.
fn adv_hud(r: &Run, input: &Input, seats: &[Ctl], cam: &Camera, lay: &Lay, area: Rect, buf: &mut Buffer) {
    let g = &r.game;
    let secs = r.left / HZ;
    let low = secs < 30 && (g.steps / 15) % 2 == 0;
    let label = format!("TIME {}:{:02}", secs / 60, secs % 60);
    let col = if low { [255, 70, 40] } else { [230, 230, 236] };
    if lay.bar == 3 {
        draw_bar(buf, area, g.steps, &format!("{} ", r.code()), &r.stage.name, &label, col);
    } else {
        let line = format!(" uXo   {} {}   {label}", r.code(), r.stage.name);
        Paragraph::new(line).style(Style::default().fg(AMBER)).render(Rect::new(area.x, area.y, area.width, 1), buf);
    }
    let players = r.players.min(seats.len());
    if let Some(row) = lay.players_row {
        let mut spans = Vec::new();
        for (i, &ctl) in seats.iter().enumerate().take(players) {
            let p = &g.players[i];
            spans.push(Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(player_color(i))));
            spans.push(Span::styled(format!(" {} ", "<3 ".repeat(r.lives[i] as usize).trim_end()), Style::default().fg(Color::Rgb(235, 60, 80))));
            spans.push(Span::styled(format!(" {:>7}  ", r.score[i]), Style::default().fg(AMBER)));
            let state = if r.lives[i] == 0 { "out of lives".to_string() } else if !input.connected(ctl) { "pad unplugged".into() } else { kit(p) };
            spans.push(Span::styled(format!("{state}   "), Style::default().fg(Color::Rgb(220, 220, 225))));
        }
        Paragraph::new(Line::from(spans).centered()).render(Rect::new(area.x, row, area.width, 1), buf);
    }
    if lay.cards {
        let card_w = lay.side - 4;
        let card_h = (lay.play.height / 2).min(17);
        let spots = [(area.x + 2, lay.play.y), (area.right() - lay.side + 2, lay.play.y)];
        for (i, &ctl) in seats.iter().enumerate().take(players.min(2)) {
            let (x, y) = spots[i];
            adv_card(r, input, i, ctl, Rect::new(x, y, card_w, card_h), area, buf);
        }
    }
    let keys = if input.exact() { "keys: exact" } else { "keys: one at a time" };
    let hint = format!(
        "bomb: A X Space Enter · action: B F / · ride: Y E . · pause: Start P · size: +/- 0 ({}) · title: Esc · {keys}",
        cam.label()
    );
    if area.height >= 3 {
        Paragraph::new(Line::from(hint).centered()).style(Style::default().fg(DIM)).render(Rect::new(area.x, area.bottom() - 2, area.width, 1), buf);
    }
}

/// An adventure card: hero, lives, score, what you ride, stats and badges.
fn adv_card(r: &Run, input: &Input, i: usize, ctl: Ctl, rect: Rect, area: Rect, buf: &mut Buffer) {
    let g = &r.game;
    let p = &g.players[i];
    let col = player_color(i);
    let title = Line::from(vec![
        Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(col).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {} ", input.label(ctl)), Style::default().fg(col)),
    ]);
    let block = Block::bordered().border_type(BorderType::Rounded).border_style(Style::default().fg(col)).title(title);
    let inner = block.inner(rect);
    Clear.render(rect, buf);
    block.render(rect, buf);
    if inner.height < 6 {
        return;
    }
    render::hero_badge(buf, inner.x + 1, inner.y, i, p, g.steps, area);
    let white = Style::default().fg(Color::Rgb(225, 225, 230));
    let dim = Style::default().fg(DIM);
    let lives = if r.lives[i] == 0 { "out".to_string() } else { "<3 ".repeat(r.lives[i] as usize).trim_end().to_string() };
    let mut right = vec![
        Line::from(Span::styled(lives, Style::default().fg(Color::Rgb(235, 60, 80)).add_modifier(Modifier::BOLD))),
        Line::from(vec![Span::styled("score ", dim), Span::styled(format!("{}", r.score[i]), Style::default().fg(AMBER))]),
    ];
    if let Some(m) = p.mount {
        right.push(Line::from(vec![Span::styled("riding ", dim), Span::styled(g.species[m].name.clone(), white)]));
    }
    if !p.spare.is_empty() {
        right.push(Line::from(vec![Span::styled("eggs ", dim), Span::styled(format!("{}", p.spare.len()), white)]));
    }
    Paragraph::new(right).render(Rect::new(inner.x + 11, inner.y, inner.width.saturating_sub(11), 4), buf);
    let stats = Line::from(vec![
        Span::styled("bombs ", dim),
        Span::styled(format!("{}", p.bombs), white),
        Span::styled(" fire ", dim),
        Span::styled(format!("{}", p.fire), white),
        Span::styled(" speed ", dim),
        Span::styled(format!("{}", p.speed), white),
    ]);
    Paragraph::new(stats).render(Rect::new(inner.x + 1, inner.y + 5, inner.width.saturating_sub(1), 1), buf);
    let room = inner.height.saturating_sub(7) / 3;
    for (n, item) in badges(p).into_iter().enumerate().take(room as usize * 3) {
        let (bx, by) = (inner.x + 1 + (n as u16 % 3) * 7, inner.y + 7 + (n as u16 / 3) * 3);
        render::item_badge(buf, bx, by, item, area);
    }
}

/// The adventure's cards and banners: story, clear, time up, game over, ending.
fn adv_banner(r: &Run, area: Rect, buf: &mut Buffer) {
    use adventure::Phase as A;
    let gold = Style::default().fg(AMBER).add_modifier(Modifier::BOLD);
    let white = Style::default().fg(Color::Rgb(230, 230, 236));
    let dim = Style::default().fg(DIM);
    // type the story out a letter at a time
    let typed = |lines: &[String], n: f32| -> Vec<Line<'static>> {
        let mut left = n as usize;
        lines
            .iter()
            .map(|l| {
                let k = l.chars().count().min(left);
                left -= k;
                Line::from(Span::styled(l.chars().take(k).collect::<String>(), white))
            })
            .collect()
    };
    match r.phase {
        A::Card(n) => {
            let mut lines = vec![
                Line::from(Span::styled(format!("{}  {}", r.stage.world.to_uppercase(), r.stage.clock), dim)).centered(),
                Line::from(Span::styled(format!("{}  {}", r.code(), r.stage.name.to_uppercase()), gold)).centered(),
                Line::raw(""),
            ];
            lines.extend(typed(&r.card_lines(), n));
            lines.push(Line::raw(""));
            let goal = match r.game.goal {
                game::Goal::Clear => "Goal: beat every baddie, then reach the hatch.",
                game::Goal::Duds => "Goal: set off every charge, then reach the hatch.",
                game::Goal::Exit => "Goal: find your way to the hatch.",
                game::Goal::Boss => "Goal: beat the guardian.",
                game::Goal::None => "",
            };
            lines.push(Line::from(Span::styled(goal, Style::default().fg(Color::Rgb(255, 220, 90)))).centered());
            lines.push(Line::from(Span::styled("bomb to begin", dim)).centered());
            panel(lines, area, buf);
        }
        A::Clear { .. } => {
            let t = &r.tally;
            let mut lines = vec![Line::from(Span::styled("STAGE CLEAR!", gold)).centered(), Line::raw("")];
            lines.push(Line::from(vec![Span::styled(format!("{:<20}", "baddies beaten"), dim), Span::styled(format!("{:>8}", t.baddies), white)]));
            if t.duds > 0 {
                lines.push(Line::from(vec![Span::styled(format!("{:<20}", "charges set off"), dim), Span::styled(format!("{:>8}", t.duds), white)]));
            }
            lines.push(Line::from(vec![Span::styled(format!("{:<20}", "time bonus"), dim), Span::styled(format!("{:>8}", t.time_bonus), white)]));
            lines.push(Line::raw(""));
            for i in 0..r.players {
                lines.push(Line::from(vec![
                    Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(player_color(i))),
                    Span::styled(format!("  score {:>9}", r.score[i]), Style::default().fg(AMBER)),
                ]));
            }
            panel(lines, area, buf);
        }
        A::TimeUp { .. } => banner(&["TIME UP!", "the stage starts again"], area, buf),
        A::GameOver => {
            let mut lines = vec![Line::from(Span::styled("GAME OVER", gold)).centered(), Line::raw("")];
            for i in 0..r.players {
                lines.push(Line::from(vec![
                    Span::styled(format!(" P{} ", i + 1), Style::default().fg(Color::Black).bg(player_color(i))),
                    Span::styled(format!("  score {:>9}", r.score[i]), Style::default().fg(AMBER)),
                ]));
            }
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(format!("bomb to try {} again · Esc for the title", r.code()), dim)).centered());
            panel(lines, area, buf);
        }
        A::Ending(n) => {
            let mut lines = vec![Line::from(Span::styled(format!("{} CLEARED", r.stage.world.to_uppercase()), gold)).centered(), Line::raw("")];
            lines.extend(typed(&r.card_lines(), n));
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled("bomb to go on", dim)).centered());
            panel(lines, area, buf);
        }
        A::Done => banner(&["TO BE CONTINUED", "act 2, the Forgotten Game, is next", "bomb for the title"], area, buf),
        A::Play => {}
    }
}

/// Headless: stage a battle scene (four players, items about, a blast, a kicked bomb,
/// a punched one in the air, a rider on the border, a sudden-death block falling),
/// run `--secs`, and paint the terminal cells into a PPM (8x16 pixels per cell).
fn snapshot(args: &[String], out: &str) -> io::Result<()> {
    use game::{Dir, Motion, Tile, FUSE, H, W};
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let (cols, rows) = arg("--size")
        .and_then(|s| s.split_once('x').map(|(a, b)| (a.parse().unwrap_or(120), b.parse().unwrap_or(42))))
        .unwrap_or((120u16, 42u16));
    let secs: f32 = arg("--secs").and_then(|s| s.parse().ok()).unwrap_or(0.3);
    let area = Rect::new(0, 0, cols, rows);
    let seats: Vec<Ctl> = vec![Ctl::Keys(0), Ctl::Keys(1)];
    let input = Input::new(false);
    // --title, or --adventure N [--card]: the title screen, or adventure stage N
    if args.iter().any(|a| a == "--title") {
        let mut t = Title::new(7);
        for _ in 0..(secs * HZ as f32) as usize {
            title_step(&mut t, &[Cmd::default(); MAX_PLAYERS], 0);
        }
        let mut buf = Buffer::empty(area);
        let mut cam = Camera::new();
        let full = Rect::new(0, 0, cols, rows.saturating_sub(2));
        cam.update(&t.game, full, 0.0, true);
        render::draw_arena(&t.game, cam.zoom, cam.at(), 0, full, &mut buf);
        render::dim(&mut buf, full, 0.4);
        title_screen(&t, &input, &seats[..1], area, &mut buf);
        return write_ppm(&buf, out);
    }
    if let Some(n) = arg("--adventure").and_then(|v| v.parse::<usize>().ok()) {
        let mut r = Run::new(7, 2, n.saturating_sub(1));
        if !args.iter().any(|a| a == "--card") {
            r.phase = adventure::Phase::Play;
            for _ in 0..(secs * HZ as f32) as usize {
                r.step(&[Cmd::default(); MAX_PLAYERS]);
            }
        } else {
            r.phase = adventure::Phase::Card(999.0);
        }
        if let Some(sp) = arg("--ride") {
            let k = r.game.species.iter().position(|s| s.id == sp).unwrap_or(0);
            r.game.players[0].mount = Some(k);
            r.game.players[0].spare = vec![k];
        }
        let lay = Lay::of(area, 2);
        let mut cam = Camera::new();
        cam.manual = arg("--zoom").and_then(|z| Zoom::parse(&z));
        cam.update(&r.game, lay.play, 0.0, true);
        let mut buf = Buffer::empty(area);
        render::draw_arena(&r.game, cam.zoom, cam.at(), 0, lay.play, &mut buf);
        adv_hud(&r, &input, &seats, &cam, &lay, area, &mut buf);
        adv_banner(&r, area, &mut buf);
        let text: String = (0..rows).map(|y| (0..cols).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>() + "\n").collect();
        std::fs::write(format!("{out}.txt"), text)?;
        return write_ppm(&buf, out);
    }
    let mut m = Match::new(7, 4);
    if let Some(n) = arg("--stage").and_then(|v| v.parse::<usize>().ok()) {
        m.stage_no = n.saturating_sub(1) % stage::count();
        m.stage = stage::load(m.stage_no);
        m.game = Game::from_stage(&m.stage, 7);
        for i in 0..MAX_PLAYERS {
            m.game.join(i);
        }
    } else {
        m.stage_no = 0;
        m.stage = stage::load(0);
        m.game = Game::new(7);
        for i in 0..MAX_PLAYERS {
            m.game.join(i);
        }
    }
    m.game.battle = true;
    m.phase = Phase::Fight;
    m.wins = [2, 1, 0, 1];
    m.round = 5;
    let staged = arg("--stage").is_none();
    if staged {
        let g = &mut m.game;
        // a cross through the middle so the blast has room to show
        for x in 1..W - 1 {
            if g.tiles[5][x] == Tile::Soft {
                g.tiles[5][x] = Tile::Floor;
            }
        }
        for y in 1..H - 1 {
            if g.tiles[y][7] == Tile::Soft {
                g.tiles[y][7] = Tile::Floor;
            }
        }
        for (x, y) in [(3, 5), (11, 7), (3, 1), (5, 1), (1, 3), (9, 9), (11, 9), (13, 9), (13, 7), (3, 9), (1, 7)] {
            g.tiles[y][x] = Tile::Floor;
        }
        g.players[0].fire = 3;
        g.place_bomb(0, 7, 5);
        g.bombs[0].fuse = 2;
        g.place_bomb(1, 3, 5);
        g.bombs[1].kind = BombKind::Remote;
        g.place_bomb(2, 11, 7);
        g.bombs[2].fuse = FUSE / 4;
        // a kicked bomb sliding along row 9, and a punched one in the air
        g.place_bomb(1, 9, 9);
        g.bombs[3].motion = Motion::Slide { dx: 1, dy: 0, speed: 40 };
        g.bombs[3].fuse = 999;
        g.place_bomb(3, 3, 9);
        let k = g.bombs.len() - 1;
        g.bombs[k].fuse = 999;
        g.bombs[k].motion = Motion::Fly { from: (3 * 256 + 128, 9 * 256 + 128), to: (6 * 256 + 128, 9 * 256 + 128), t: 9, dur: 18, dx: 1, dy: 0 };
        let items = [Item::BombUp, Item::FireUp, Item::SpeedUp, Item::Kick, Item::Glove, Item::Skull, Item::Heart, Item::Remote, Item::Pierce, Item::Vest];
        for (n, (x, y)) in [(3, 1), (5, 1), (1, 3), (13, 7), (1, 7), (13, 9), (1, 9), (3, 3), (5, 3), (9, 1)].into_iter().enumerate() {
            g.tiles[y][x] = Tile::Floor;
            g.items[y][x] = Some(items[n]);
        }
        g.players[0].facing = Dir::Right;
        g.players[0].kick = true;
        g.players[0].heart = true;
        g.players[2].facing = Dir::Left;
        g.players[2].disease = Some(game::Disease::Reverse);
        g.players[2].disease_left = 999;
        // player 4 knocked out and riding the border
        g.players[3].out = true;
        g.players[3].alive = false;
        g.players[3].rider = Some(256 * 45);
        g.tiles[11][2] = Tile::Pressure(4);
    }
    // --surge: an open room, a surge flooding round a wall and up to a soft block
    if args.iter().any(|a| a == "--surge") {
        let g = &mut m.game;
        g.bombs.clear();
        for y in 1..H - 1 {
            for x in 1..W - 1 {
                g.tiles[y][x] = Tile::Floor;
                g.items[y][x] = None;
            }
        }
        for (x, y) in [(9, 5), (9, 6), (9, 7), (5, 3)] {
            g.tiles[y][x] = Tile::Wall;
        }
        g.tiles[9][7] = Tile::Soft;
        for (i, (x, y)) in [(1, 1), (13, 1), (1, 11), (13, 11)].into_iter().enumerate() {
            g.players[i].x = game::centre(x);
            g.players[i].y = game::centre(y);
            g.players[i].out = false;
            g.players[i].alive = true;
            g.players[i].rider = None;
        }
        g.players[0].kind = BombKind::Surge;
        g.players[0].fire = 8;
        g.place_bomb(0, 7, 6);
        g.bombs[0].fuse = 1;
    }
    // --puzzle: every puzzle tile in an open room, one block on the move
    if args.iter().any(|a| a == "--puzzle") {
        use game::{Ground, Pickup};
        let g = &mut m.game;
        g.bombs.clear();
        for y in 1..H - 1 {
            for x in 1..W - 1 {
                g.tiles[y][x] = Tile::Floor;
                g.items[y][x] = None;
            }
        }
        for (x, y) in [(3, 3), (4, 3), (6, 8)] {
            g.tiles[y][x] = Tile::Block;
        }
        g.pickups[3][6] = Some(Pickup::Key(0));
        g.pickups[3][7] = Some(Pickup::Key(1));
        g.tiles[3][9] = Tile::Door(0);
        g.tiles[3][10] = Tile::Door(1);
        g.ground[5][3] = Ground::Switch;
        g.ground[5][5] = Ground::Switch;
        g.tiles[5][5] = Tile::Block;
        for x in 8..11 {
            g.tiles[5][x] = Tile::Gate;
            g.ground[5][x] = Ground::Gate;
        }
        g.pickups[9][3] = Some(Pickup::Part);
        g.pickups[9][5] = Some(Pickup::Part);
        g.tiles[9][10] = Tile::Socket;
        for x in 9..12 {
            g.tiles[7][x] = Tile::Wall;
            g.ground[7][x] = Ground::Water;
        }
        g.tiles[7][9] = Tile::Floor;
        g.ground[7][9] = Ground::Sunk;
        for (i, (x, y)) in [(1, 1), (13, 1), (1, 11), (13, 11)].into_iter().enumerate() {
            g.players[i].x = game::centre(x);
            g.players[i].y = game::centre(y);
            g.players[i].out = false;
            g.players[i].alive = true;
            g.players[i].rider = None;
        }
        // two UXOs, one about to go
        g.players[0].bombs = 3;
        for (x, y, left) in [(12, 3, 400), (12, 9, 30)] {
            g.place_bomb(0, x, y);
            g.bombs.last_mut().unwrap().uxo = Some(left);
        }
        // player 1 leaning on the block at 6,8, about to push it right
        g.players[0].x = game::centre(5);
        g.players[0].y = game::centre(8);
        g.players[0].facing = Dir::Right;
        let mut push = [Cmd::default(); MAX_PLAYERS];
        push[0].primary = Some(Dir::Right);
        for _ in 0..16 {
            g.step(&push);
        }
    }
    let idle = [Cmd::default(); MAX_PLAYERS];
    for _ in 0..(secs * HZ as f32) as usize {
        m.game.step(&idle);
    }
    let area = Rect::new(0, 0, cols, rows);
    let mut buf = Buffer::empty(area);
    let lay = Lay::of(area, MAX_PLAYERS);
    let mut cam = Camera::new();
    cam.manual = arg("--zoom").and_then(|z| Zoom::parse(&z));
    cam.update(&m.game, lay.play, 0.0, true);
    let t0 = Instant::now();
    for _ in 0..20 {
        render::draw_arena(&m.game, cam.zoom, cam.at(), 0, lay.play, &mut buf);
    }
    eprintln!("draw: {:.3} ms per frame at {cols}x{rows}, zoom {}", t0.elapsed().as_secs_f64() * 1000.0 / 20.0, cam.label());
    let input = Input::new(false);
    let seats: Vec<Ctl> = (0..MAX_PLAYERS).map(|i| Ctl::Keys(i % 2)).collect();
    hud(&m, &input, &seats, &cam, &lay, area, &mut buf);
    if let Some(p) = arg("--banner") {
        m.tally = [
            battle::Tally { kills: 5, outs: 2, blocks: 41, items: 12, bombs: 60 },
            battle::Tally { kills: 2, outs: 3, blocks: 30, items: 9, bombs: 44 },
            battle::Tally { kills: 1, outs: 4, blocks: 18, items: 4, bombs: 31 },
            battle::Tally { kills: 3, outs: 3, blocks: 22, items: 7, bombs: 38 },
        ];
        m.phase = match p.as_str() {
            "ready" => Phase::Ready(10),
            "round" => Phase::RoundOver { winner: Some(0), t: 10 },
            "lobby" => Phase::Lobby(2),
            _ => Phase::MatchOver { winner: 0 },
        };
        phase_banner(&m, area, &mut buf);
    }
    let text: String = (0..rows)
        .map(|r| (0..cols).map(|c| buf[(c, r)].symbol().to_string()).collect::<String>() + "\n")
        .collect();
    std::fs::write(format!("{out}.txt"), text)?;
    write_ppm(&buf, out)
}

fn write_ppm(buf: &Buffer, out: &str) -> io::Result<()> {
    let (cw, ch) = (8usize, 16usize);
    let (cols, rows) = (buf.area.width as usize, buf.area.height as usize);
    let (w, h) = (cols * cw, rows * ch);
    let mut img = vec![0u8; w * h * 3];
    let rgb = |c: Color| match c {
        Color::Rgb(r, g, b) => [r, g, b],
        Color::Black => [0, 0, 0],
        _ => [200, 200, 200],
    };
    for r in 0..rows {
        for c in 0..cols {
            let cell = &buf[(c as u16, r as u16)];
            let (fg, bg) = (rgb(cell.fg), if cell.bg == Color::Reset { [0, 0, 0] } else { rgb(cell.bg) });
            let sym = cell.symbol().chars().next().unwrap_or(' ');
            for y in 0..ch {
                for x in 0..cw {
                    let col = match sym {
                        '▀' => if y < ch / 2 { fg } else { bg },
                        '█' => fg,
                        '▒' | '▓' => if (x + y) % 2 == 0 { fg } else { bg },
                        ' ' => bg,
                        _ => if (2..6).contains(&x) && (4..12).contains(&y) { fg } else { bg },
                    };
                    let i = ((r * ch + y) * w + c * cw + x) * 3;
                    img[i..i + 3].copy_from_slice(&col);
                }
            }
        }
    }
    let mut data = format!("P6\n{w} {h}\n255\n").into_bytes();
    data.extend_from_slice(&img);
    std::fs::write(out, data)
}
