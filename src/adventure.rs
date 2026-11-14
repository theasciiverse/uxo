//! Adventure: the worlds, stage by stage, alone or two together. A story card before
//! each stage; lives and score; a clock per stage; the exit opens when the stage's goal
//! is met (beat every baddie, or set off every dud), or the boss falls. Power-ups and
//! mounts carry on to the next stage; a lost life costs the handling items.

use crate::game::{Cmd, Event, Game, HZ, MAX_PLAYERS};
use crate::stage::{self, Stage};
use log::info;

pub const STAGES: [&str; 90] = [
    include_str!("../adventure/orchard/01-windfall-lane.txt"),
    include_str!("../adventure/orchard/02-beetle-rows.txt"),
    include_str!("../adventure/orchard/03-hedge-maze.txt"),
    include_str!("../adventure/orchard/04-frog-pond.txt"),
    include_str!("../adventure/orchard/05-cider-press.txt"),
    include_str!("../adventure/orchard/06-ram-pasture.txt"),
    include_str!("../adventure/orchard/07-crow-tree.txt"),
    include_str!("../adventure/orchard/08-the-old-grove.txt"),
    include_str!("../adventure/orchard/09-bramblehog.txt"),
    include_str!("../adventure/clock/01-the-second-hand.txt"),
    include_str!("../adventure/clock/02-tick-and-tock.txt"),
    include_str!("../adventure/clock/03-gear-train.txt"),
    include_str!("../adventure/clock/04-the-escapement.txt"),
    include_str!("../adventure/clock/05-the-alarm-room.txt"),
    include_str!("../adventure/clock/06-the-cog-drawer.txt"),
    include_str!("../adventure/clock/07-the-pendulum.txt"),
    include_str!("../adventure/clock/08-the-clock-tower.txt"),
    include_str!("../adventure/clock/09-the-cuckoo.txt"),
    include_str!("../adventure/buffer/01-write-queue.txt"),
    include_str!("../adventure/buffer/02-packet-lanes.txt"),
    include_str!("../adventure/buffer/03-null-pointer.txt"),
    include_str!("../adventure/buffer/04-overflow.txt"),
    include_str!("../adventure/buffer/05-cache-lines.txt"),
    include_str!("../adventure/buffer/06-the-flush.txt"),
    include_str!("../adventure/buffer/07-ring-buffer.txt"),
    include_str!("../adventure/buffer/08-the-last-charges.txt"),
    include_str!("../adventure/buffer/09-purged.txt"),
    // act 2, the Forgotten Game
    include_str!("../adventure/cold/01-thaw.txt"),
    include_str!("../adventure/cold/02-deep-freeze.txt"),
    include_str!("../adventure/cold/03-the-zip-lines.txt"),
    include_str!("../adventure/cold/04-packed-rooms.txt"),
    include_str!("../adventure/cold/05-the-skating-hall.txt"),
    include_str!("../adventure/cold/06-the-archive.txt"),
    include_str!("../adventure/cold/07-snowdrift.txt"),
    include_str!("../adventure/cold/08-the-long-sleep.txt"),
    include_str!("../adventure/cold/09-the-freezer.txt"),
    include_str!("../adventure/bin/01-deleted-items.txt"),
    include_str!("../adventure/bin/02-paper-heaps.txt"),
    include_str!("../adventure/bin/03-the-sort.txt"),
    include_str!("../adventure/bin/04-old-saves.txt"),
    include_str!("../adventure/bin/05-the-shortcuts.txt"),
    include_str!("../adventure/bin/06-lost-and-found.txt"),
    include_str!("../adventure/bin/07-the-chute.txt"),
    include_str!("../adventure/bin/08-empty-bin.txt"),
    include_str!("../adventure/bin/09-the-compactor.txt"),
    include_str!("../adventure/games/01-press-any-key.txt"),
    include_str!("../adventure/games/02-the-high-scores.txt"),
    include_str!("../adventure/games/03-bonus-round.txt"),
    include_str!("../adventure/games/04-the-warp-zone.txt"),
    include_str!("../adventure/games/05-two-player-mode.txt"),
    include_str!("../adventure/games/06-the-secret-level.txt"),
    include_str!("../adventure/games/07-hard-mode.txt"),
    include_str!("../adventure/games/08-the-last-level.txt"),
    include_str!("../adventure/games/09-demo.txt"),
    // act 3, the Virus
    include_str!("../adventure/power/01-live-wires.txt"),
    include_str!("../adventure/power/02-the-rectifier.txt"),
    include_str!("../adventure/power/03-fan-duct.txt"),
    include_str!("../adventure/power/04-capacitor-bank.txt"),
    include_str!("../adventure/power/05-the-arc-gap.txt"),
    include_str!("../adventure/power/06-switch-mode.txt"),
    include_str!("../adventure/power/07-the-heat-sink.txt"),
    include_str!("../adventure/power/08-overload.txt"),
    include_str!("../adventure/power/09-the-big-fan.txt"),
    include_str!("../adventure/scroll/01-last-login.txt"),
    include_str!("../adventure/scroll/02-tail-end.txt"),
    include_str!("../adventure/scroll/03-the-history.txt"),
    include_str!("../adventure/scroll/04-grep.txt"),
    include_str!("../adventure/scroll/05-cat-on-the-keys.txt"),
    include_str!("../adventure/scroll/06-pipes.txt"),
    include_str!("../adventure/scroll/07-page-down.txt"),
    include_str!("../adventure/scroll/08-buffer-full.txt"),
    include_str!("../adventure/scroll/09-the-pager.txt"),
    include_str!("../adventure/static/01-snow.txt"),
    include_str!("../adventure/static/02-glitches.txt"),
    include_str!("../adventure/static/03-the-typo.txt"),
    include_str!("../adventure/static/04-interference.txt"),
    include_str!("../adventure/static/05-test-pattern.txt"),
    include_str!("../adventure/static/06-dead-pixels.txt"),
    include_str!("../adventure/static/07-signal-lost.txt"),
    include_str!("../adventure/static/08-white-noise.txt"),
    include_str!("../adventure/static/09-payload.txt"),
    include_str!("../adventure/prompt/01-the-prompt.txt"),
    include_str!("../adventure/prompt/02-arguments.txt"),
    include_str!("../adventure/prompt/03-wildcards.txt"),
    include_str!("../adventure/prompt/04-aliases.txt"),
    include_str!("../adventure/prompt/05-root.txt"),
    include_str!("../adventure/prompt/06-history.txt"),
    include_str!("../adventure/prompt/07-tab-completion.txt"),
    include_str!("../adventure/prompt/08-the-command.txt"),
    include_str!("../adventure/prompt/09-payload.txt"),
];

/// Stages in each world, in order: act 1's Screensaver, Clock and Buffer, act 2's
/// Cold Storage, Recycle Bin and Games Folder, act 3's Power Supply, Scrollback and
/// Static, and the Prompt, the last.
pub const WORLDS: [usize; 10] = [9, 9, 9, 9, 9, 9, 9, 9, 9, 9];

/// The first stage index of the world stage `index` is in, and its stage count.
pub fn world_of(index: usize) -> (usize, usize) {
    let mut first = 0;
    for &n in WORLDS.iter() {
        if index < first + n {
            return (first, n);
        }
        first += n;
    }
    (first, 1)
}

/// The stage index of a code like "2-3", for `--from`.
pub fn index_of(code: &str) -> Option<usize> {
    let (w, n) = code.split_once('-')?;
    let (w, n): (usize, usize) = (w.trim().parse().ok()?, n.trim().parse().ok()?);
    if w == 0 || n == 0 || w > WORLDS.len() || n > WORLDS[w - 1] {
        return None;
    }
    Some(WORLDS[..w - 1].iter().sum::<usize>() + n - 1)
}

/// "2-3": world and stage number of stage `index`.
pub fn code_of(index: usize) -> String {
    let mut first = 0;
    for (w, &n) in WORLDS.iter().enumerate() {
        if index < first + n {
            return format!("{}-{}", w + 1, index - first + 1);
        }
        first += n;
    }
    format!("?-{}", index + 1)
}

pub fn load(i: usize) -> Stage {
    stage::parse(STAGES[i]).unwrap_or_else(|e| panic!("adventure stage {}: {e}", i + 1))
}

pub const LIVES: u32 = 3;
const DUD_POINTS: u32 = 500;
/// Points for emptying a deleted file from the Recycle Bin.
const EMPTY_POINTS: u32 = 100;
/// Points per second left on the clock at a clear.
const TIME_BONUS: u32 = 10;
/// Characters per step the story card types.
const TYPE_SPEED: f32 = 0.8;
const CLEAR_HOLD: u32 = HZ * 4;
const TIME_UP_HOLD: u32 = HZ * 3;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    /// The story card before a stage; how much has typed out.
    Card(f32),
    Play,
    Clear { t: u32 },
    TimeUp { t: u32 },
    GameOver,
    /// After the world's last stage: its ending.
    Ending(f32),
    /// The Forgotten Game's map of the world: pick the next stage among those open.
    /// `cursor` is a stage index.
    Map { cursor: usize },
    /// Nothing more yet.
    Done,
}

/// What a stage earned, for the clear screen.
#[derive(Clone, Copy, Default, Debug)]
pub struct Tally {
    pub baddies: u32,
    pub duds: u32,
    pub emptied: u32,
    pub time_bonus: u32,
}

/// Percent of bombs that fizzle into UXOs in the adventure.
const FIZZLE: u32 = 3;

pub struct Run {
    pub game: Game,
    pub stage: Stage,
    pub index: usize,
    pub players: usize,
    pub lives: [u32; MAX_PLAYERS],
    pub score: [u32; MAX_PLAYERS],
    pub phase: Phase,
    /// Steps left on the stage clock.
    pub left: u32,
    pub tally: Tally,
    /// Stages cleared on this run, for the branching maps.
    pub cleared: Vec<bool>,
    /// The last stage was lost to Payload's command, not the clock.
    pub entered: bool,
    seed: u64,
}

impl Run {
    pub fn new(seed: u64, players: usize, start: usize) -> Run {
        let index = start.min(STAGES.len() - 1);
        let stage = load(index);
        let mut r = Run {
            game: Game::from_stage(&stage, seed),
            stage,
            index,
            players,
            lives: [LIVES; MAX_PLAYERS],
            score: [0; MAX_PLAYERS],
            phase: Phase::Card(0.0),
            left: 0,
            tally: Tally::default(),
            cleared: vec![false; STAGES.len()],
            entered: false,
            seed,
        };
        r.begin(false);
        r
    }

    /// "1-3": world and stage number.
    pub fn code(&self) -> String {
        code_of(self.index)
    }

    /// Does the current world have a branching map?
    pub fn has_map(&self) -> bool {
        self.stage.map.is_some()
    }

    /// The stages open on the current world's map: the first, the one being played,
    /// and every stage a cleared one leads to. Cleared stages stay open (to play again).
    pub fn open_stages(&self) -> Vec<usize> {
        let (first, n) = world_of(self.index);
        let mut open = vec![false; n];
        open[0] = true;
        open[self.index - first] = true;
        for i in first..first + n {
            if self.cleared[i] {
                open[i - first] = true;
                for &k in &load(i).next {
                    if k >= 1 && k <= n {
                        open[k - 1] = true;
                    }
                }
            }
        }
        (0..n).filter(|&k| open[k]).map(|k| first + k).collect()
    }

    /// Where the map cursor starts: the first open stage not yet cleared that the
    /// stage just cleared leads to.
    fn map_start(&self) -> usize {
        let (first, _) = world_of(self.index);
        let open = self.open_stages();
        load(self.index)
            .next
            .iter()
            .map(|k| first + k - 1)
            .find(|i| open.contains(i) && !self.cleared[*i])
            .or_else(|| open.iter().copied().find(|&i| !self.cleared[i]))
            .unwrap_or(self.index)
    }

    /// Move the map cursor one step toward `d`: the nearest open stage that way.
    pub fn map_move(&mut self, d: crate::game::Dir) {
        let Phase::Map { cursor } = self.phase else { return };
        let at = load(cursor).map.unwrap_or((0, 0));
        let (dx, dy) = d.vec();
        let best = self
            .open_stages()
            .into_iter()
            .filter(|&i| i != cursor)
            .filter_map(|i| {
                let m = load(i).map?;
                let (ex, ey) = (m.0 as i32 - at.0 as i32, m.1 as i32 - at.1 as i32);
                let along = ex * dx + ey * dy;
                let side = (ex * dy - ey * dx).abs();
                (along > 0).then_some((along * 2 + side * 3, i))
            })
            .min();
        if let Some((_, i)) = best {
            self.phase = Phase::Map { cursor: i };
        }
    }

    /// Set up the current stage. `carry` keeps each player's power-ups and mount.
    fn begin(&mut self, carry: bool) {
        let old: Vec<crate::game::Player> = self.game.players.to_vec();
        self.stage = load(self.index);
        let seed = self.seed ^ (self.index as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        self.game = Game::from_stage(&self.stage, seed);
        self.game.adventure = true;
        self.game.hurry_at = u32::MAX;
        // now and then a bomb fizzles: a surprise, not a rule
        self.game.fizzle = FIZZLE;
        for i in 0..self.players {
            if self.lives[i] == 0 {
                continue;
            }
            self.game.join(i);
            if carry && old[i].joined {
                let (o, p) = (&old[i], &mut self.game.players[i]);
                p.bombs = o.bombs;
                p.fire = o.fire;
                p.speed = o.speed;
                p.kick = o.kick;
                p.punch = o.punch;
                p.glove = o.glove;
                p.line = o.line;
                p.kind = o.kind;
                p.bomb_pass = o.bomb_pass;
                p.block_pass = o.block_pass;
                p.heart = o.heart;
                p.mount = o.mount;
                p.spare = o.spare.clone();
                // what the stage hands out comes on top of what was carried
                self.game.give_items(i);
            }
        }
        self.left = self.stage.time * HZ;
        self.tally = Tally::default();
        self.entered = false;
        self.phase = Phase::Card(0.0);
        info!("playtest {} {}: start, lives {}", self.code(), self.stage.name, self.lives_text());
    }

    /// Lives left per player, for the playtest lines: "3" or "3/2".
    fn lives_text(&self) -> String {
        self.lives[..self.players].iter().map(u32::to_string).collect::<Vec<_>>().join("/")
    }

    /// Seconds played on this stage so far.
    fn played(&self) -> u32 {
        (self.stage.time * HZ).saturating_sub(self.left) / HZ
    }

    /// A second player joins in at once, with fresh lives.
    pub fn add_player(&mut self) -> usize {
        let i = self.players;
        self.players += 1;
        self.lives[i] = LIVES;
        if i < 2 || self.game.spawns.len() > i {
            self.game.join(i);
            self.game.players[i].invuln = crate::game::INVULN;
        }
        i
    }

    pub fn card_lines(&self) -> Vec<String> {
        let lines = match self.phase {
            Phase::Ending(_) => &self.stage.ending,
            _ => &self.stage.story,
        };
        // `{dot}` names the heroes: Dot alone, Dot and Caret together
        let heroes = if self.players > 1 { "Dot and Caret" } else { "Dot" };
        lines.iter().map(|l| l.replace("{dot}", heroes)).collect()
    }

    pub fn step(&mut self, cmds: &[Cmd; MAX_PLAYERS]) -> Vec<Event> {
        let bomb = cmds.iter().any(|c| c.bomb);
        match self.phase {
            Phase::Card(typed) | Phase::Ending(typed) => {
                let total: usize = self.card_lines().iter().map(|l| l.chars().count()).sum();
                let typed = typed + TYPE_SPEED;
                let done = typed as usize >= total;
                let ending = matches!(self.phase, Phase::Ending(_));
                self.phase = if ending { Phase::Ending(typed) } else { Phase::Card(typed) };
                if bomb {
                    if !done {
                        // the first press finishes the typing
                        self.phase = if ending { Phase::Ending(total as f32) } else { Phase::Card(total as f32) };
                    } else if ending && self.index + 1 < STAGES.len() {
                        // on into the next world, keeping what was carried
                        self.index += 1;
                        self.begin(true);
                    } else if ending {
                        self.phase = Phase::Done;
                    } else {
                        self.phase = Phase::Play;
                    }
                }
            }
            Phase::Play => {
                self.game.step(cmds);
                self.left = self.left.saturating_sub(1);
                if self.left == 0 {
                    info!("playtest {} {}: time up", self.code(), self.stage.name);
                    for i in 0..self.players {
                        if self.game.players[i].joined && !self.game.players[i].out {
                            self.lose_life(i);
                        }
                    }
                    self.phase = Phase::TimeUp { t: TIME_UP_HOLD };
                }
            }
            Phase::Clear { t } => {
                // the fireworks keep going while the tally shows
                self.game.step(&[Cmd::default(); MAX_PLAYERS]);
                if t > 1 {
                    self.phase = Phase::Clear { t: t - 1 };
                } else if self.has_map() && self.stage.ending.is_empty() {
                    self.phase = Phase::Map { cursor: self.map_start() };
                } else if self.index + 1 < STAGES.len() && self.stage.ending.is_empty() {
                    self.index += 1;
                    self.begin(true);
                } else {
                    self.phase = Phase::Ending(0.0);
                }
            }
            Phase::TimeUp { t } => {
                if t > 1 {
                    self.phase = Phase::TimeUp { t: t - 1 };
                } else if self.all_out() {
                    self.phase = Phase::GameOver;
                } else {
                    self.begin(true);
                }
            }
            Phase::GameOver => {
                if bomb {
                    // try the stage again, fresh
                    self.lives = [LIVES; MAX_PLAYERS];
                    self.score = [0; MAX_PLAYERS];
                    self.begin(false);
                }
            }
            Phase::Map { cursor } => {
                for c in cmds {
                    if let Some(d) = c.tap {
                        self.map_move(d);
                    }
                }
                if bomb {
                    info!("playtest map: {} chosen", code_of(cursor));
                    self.index = cursor;
                    self.begin(true);
                }
            }
            Phase::Done => {}
        }
        let events = std::mem::take(&mut self.game.events);
        for &ev in &events {
            self.score_event(ev);
        }
        events
    }

    /// Start the stage over from the pause screen, for a life each. False when not
    /// playing a stage.
    pub fn start_over(&mut self) -> bool {
        if self.phase != Phase::Play {
            return false;
        }
        info!("playtest {} {}: started over after {}s", self.code(), self.stage.name, self.played());
        for i in 0..self.players {
            if self.game.players[i].joined && !self.game.players[i].out {
                self.lose_life(i);
            }
        }
        if self.all_out() {
            self.phase = Phase::GameOver;
        } else {
            self.begin(true);
        }
        true
    }

    fn all_out(&self) -> bool {
        (0..self.players).all(|i| self.lives[i] == 0)
    }

    fn lose_life(&mut self, i: usize) {
        self.lives[i] = self.lives[i].saturating_sub(1);
        if self.lives[i] == 0 {
            let p = &mut self.game.players[i];
            p.out = true;
            p.alive = false;
        }
    }

    /// What caught a player who wasn't in a blast: the creature nearest them, or a
    /// block crumbling round them.
    fn toucher(&self, player: usize) -> String {
        let p = &self.game.players[player];
        let near = self
            .game
            .baddies
            .iter()
            .filter(|b| b.alive)
            .map(|b| ((b.x - p.x).abs() + (b.y - p.y).abs(), b.species))
            .min();
        let (tx, ty) = p.tile();
        if self.game.ground[ty][tx] == crate::game::Ground::Wire {
            return "a live wire".to_string();
        }
        if self.game.ground[ty][tx] == crate::game::Ground::Scrolled {
            return "the scroll".to_string();
        }
        match near {
            Some((d, sp)) if d < 3 * crate::game::TILE => self.game.species[sp].name.clone(),
            _ => "a crumbling block".to_string(),
        }
    }

    fn score_event(&mut self, ev: Event) {
        match ev {
            Event::BaddieDown { species, by, x, y } => {
                let sp = &self.game.species[species];
                if by < MAX_PLAYERS {
                    self.score[by] += sp.points;
                }
                self.tally.baddies += 1;
                if sp.size > 1 {
                    // the boss goes up like the biggest firework of all
                    for k in 0..5 {
                        self.game.fireworks(x + (k - 2) * 120, y - (k % 2) * 160, 36);
                    }
                }
            }
            Event::Emptied { by, .. } => {
                if by < MAX_PLAYERS {
                    self.score[by] += EMPTY_POINTS;
                }
                self.tally.emptied += 1;
            }
            Event::DudPopped { by, .. } => {
                if by < MAX_PLAYERS {
                    self.score[by] += DUD_POINTS;
                }
                self.tally.duds += 1;
            }
            Event::Death { player, by } if self.phase == Phase::Play => {
                let what = match by {
                    Some(k) if k == player => "own blast".to_string(),
                    Some(crate::game::HOUSE) => "the game's bomb".to_string(),
                    Some(k) => format!("P{}'s blast", k + 1),
                    None => self.toucher(player),
                };
                info!("playtest {} {}: P{} lost a life after {}s ({what})", self.code(), self.stage.name, player + 1, self.played());
                self.lose_life(player);
                if self.all_out() {
                    info!("playtest {} {}: game over", self.code(), self.stage.name);
                    self.phase = Phase::GameOver;
                }
            }
            Event::Enter if self.phase == Phase::Play => {
                info!("playtest {} {}: Payload pressed Enter after {}s", self.code(), self.stage.name, self.played());
                for i in 0..self.players {
                    if self.game.players[i].joined && !self.game.players[i].out {
                        self.lose_life(i);
                    }
                }
                self.entered = true;
                self.phase = if self.all_out() { Phase::GameOver } else { Phase::TimeUp { t: TIME_UP_HOLD } };
            }
            Event::Cleared if self.phase == Phase::Play => {
                let secs = self.left / HZ;
                self.tally.time_bonus = secs * TIME_BONUS;
                for i in 0..self.players {
                    if self.lives[i] > 0 {
                        self.score[i] += self.tally.time_bonus;
                    }
                }
                // a celebration over the exit (or where the boss fell)
                if let Some((ex, ey)) = self.game.exit {
                    let (cx, cy) = (ex as i32 * crate::game::TILE + 128, ey as i32 * crate::game::TILE + 128);
                    self.game.fireworks(cx, cy - 300, 40);
                    self.game.fireworks(cx - 400, cy - 150, 30);
                    self.game.fireworks(cx + 400, cy - 150, 30);
                }
                info!(
                    "playtest {} {}: cleared in {}s of {}s, lives {}, score {}",
                    self.code(),
                    self.stage.name,
                    self.played(),
                    self.stage.time,
                    self.lives_text(),
                    self.score[..self.players].iter().map(u32::to_string).collect::<Vec<_>>().join("/")
                );
                crate::progress::reached(self.index + 1);
                self.cleared[self.index] = true;
                self.phase = Phase::Clear { t: CLEAR_HOLD };
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every stage runs a minute untouched (baddies hopping, summoning, charging,
    /// shutters on the beat) without a panic.
    #[test]
    fn every_stage_runs_a_minute() {
        for i in 0..STAGES.len() {
            let mut r = Run::new(3, 2, i);
            r.phase = Phase::Play;
            for _ in 0..60 * HZ {
                r.step(&[Cmd::default(); MAX_PLAYERS]);
            }
        }
    }

    /// A life lost to a creature is put down to that creature in the playtest log.
    #[test]
    fn a_life_lost_to_a_creature_names_it() {
        let mut r = Run::new(1, 1, 0);
        r.phase = Phase::Play;
        r.game.players[0].invuln = 0;
        let (x, y, sp) = (r.game.baddies[0].x, r.game.baddies[0].y, r.game.baddies[0].species);
        for _ in 0..HZ {
            if r.lives[0] < LIVES {
                break;
            }
            (r.game.players[0].x, r.game.players[0].y) = (r.game.baddies[0].x, r.game.baddies[0].y);
            r.step(&[Cmd::default(); MAX_PLAYERS]);
        }
        assert_eq!(r.lives[0], LIVES - 1, "touching the creature at {x},{y} costs a life");
        assert_eq!(r.toucher(0), r.game.species[sp].name);
    }

    #[test]
    fn a_boss_stage_waits_for_its_boss() {
        for i in 0..STAGES.len() {
            let mut r = Run::new(1, 1, i);
            if r.stage.goal != crate::game::Goal::Boss {
                continue;
            }
            r.phase = Phase::Play;
            r.step(&[Cmd::default(); MAX_PLAYERS]);
            assert_eq!(r.phase, Phase::Play, "{} is cleared before its boss is beaten", r.stage.name);
        }
    }

    #[test]
    fn worlds_add_up_and_each_ends_with_an_ending() {
        assert_eq!(WORLDS.iter().sum::<usize>(), STAGES.len());
        let mut last = 0;
        for n in WORLDS {
            last += n;
            for i in last - n..last {
                assert_eq!(!load(i).ending.is_empty(), i == last - 1, "{}: only a world's last stage has an ending", code_of(i));
            }
        }
        assert_eq!(code_of(0), "1-1");
        assert_eq!(code_of(9), "2-1");
        assert_eq!(code_of(17), "2-9");
        assert_eq!(code_of(26), "3-9");
        for i in 0..STAGES.len() {
            assert_eq!(index_of(&code_of(i)), Some(i));
        }
        assert_eq!(index_of("11-1"), None);
        assert_eq!(index_of("1-10"), None);
        assert_eq!(index_of("0-1"), None);
    }

    #[test]
    fn a_world_ending_goes_on_into_the_next_world() {
        let mut r = Run::new(1, 1, WORLDS[0] - 1);
        r.game.players[0].bombs = 4;
        r.phase = Phase::Ending(999.0);
        let press = std::array::from_fn(|i| Cmd { bomb: i == 0, ..Cmd::default() });
        r.step(&press);
        assert_eq!(r.index, WORLDS[0]);
        assert_eq!(r.phase, Phase::Card(0.0));
        assert_eq!(r.game.players[0].bombs, 4, "power-ups carry on into the new world");
    }

    #[test]
    fn every_adventure_stage_parses_and_is_connected() {
        for i in 0..STAGES.len() {
            let s = load(i);
            assert!(!s.story.is_empty(), "{} has no story", s.name);
            let g = Game::from_stage(&s, 1);
            assert!(!g.baddies.is_empty(), "{} has no baddies", s.name);
            crate::stage::assert_connected(&s);
        }
    }

    /// The hatch can be reached the honest way: soft blocks can be blown up, crates
    /// are in the way, a door opens once its key can be reached, the sockets once
    /// every part can, and gates only on a stage with a crate to hold the switch.
    #[test]
    fn every_hatch_can_be_reached() {
        use crate::stage::Cell;
        for i in 0..STAGES.len() {
            let s = load(i);
            let Some(exit) = (0..s.h).flat_map(|y| (0..s.w).map(move |x| (x, y))).find(|&(x, y)| s.cells[y][x] == Cell::Exit) else {
                continue;
            };
            let crates = s.cells.iter().flatten().any(|c| *c == Cell::Block);
            let parts = s.cells.iter().flatten().filter(|c| **c == Cell::Part).count();
            let start = (0..s.h).flat_map(|y| (0..s.w).map(move |x| (x, y))).find(|&(x, y)| s.cells[y][x] == Cell::Start(0)).unwrap();
            let warps = crate::stage::warps(&s);
            let mut keys = [false; 4];
            let mut found = 0;
            // widen the reachable area until nothing new opens up
            loop {
                let open = |c: &Cell| match c {
                    Cell::Wall | Cell::Water | Cell::Tree | Cell::Block => false,
                    Cell::Door(k) => keys[*k as usize],
                    Cell::Gate => crates,
                    Cell::Socket => found == parts,
                    _ => true,
                };
                let mut seen = vec![vec![false; s.w]; s.h];
                let mut todo = vec![start];
                while let Some((x, y)) = todo.pop() {
                    if seen[y][x] || !open(&s.cells[y][x]) {
                        continue;
                    }
                    seen[y][x] = true;
                    if let Some(k) = warps.iter().position(|&p| p == (x, y)) {
                        todo.push(warps[(k + 1) % warps.len()]);
                    }
                    todo.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
                }
                let mut got = keys;
                let mut n = 0;
                for y in 0..s.h {
                    for x in 0..s.w {
                        match s.cells[y][x] {
                            Cell::Key(k) if seen[y][x] => got[k as usize] = true,
                            Cell::Part if seen[y][x] => n += 1,
                            _ => {}
                        }
                    }
                }
                if seen[exit.1][exit.0] {
                    break;
                }
                assert!(got != keys || n != found, "{}: the hatch can't be reached", s.name);
                (keys, found) = (got, n);
            }
        }
    }

    /// A world with a map: every stage has a place, the places differ, every stage
    /// can be reached from the first, and every way through ends at the boss.
    #[test]
    fn every_map_leads_from_the_first_stage_to_the_boss() {
        let mut first = 0;
        for n in WORLDS {
            let stages: Vec<Stage> = (first..first + n).map(load).collect();
            if stages[0].map.is_some() {
                let mut places = std::collections::HashSet::new();
                for s in &stages {
                    assert!(places.insert(s.map.expect("every stage of a map world has a place")), "{}: two stages share a place", s.name);
                    assert!(s.next.iter().all(|&k| k >= 1 && k <= n), "{}: next leads out of the world", s.name);
                    assert_eq!(s.next.is_empty(), s.goal == crate::game::Goal::Boss, "{}: only the boss leads nowhere", s.name);
                }
                let mut seen = vec![false; n];
                let mut todo = vec![0];
                while let Some(k) = todo.pop() {
                    if !std::mem::replace(&mut seen[k], true) {
                        todo.extend(stages[k].next.iter().map(|j| j - 1));
                    }
                }
                assert!(seen.iter().all(|&v| v), "{}: a stage can't be reached on the map", stages[0].world);
            }
            first += n;
        }
    }

    #[test]
    fn a_cleared_stage_on_a_map_opens_the_way_on() {
        let start = WORLDS[..3].iter().sum::<usize>();
        let mut r = Run::new(1, 1, start);
        assert!(r.has_map());
        assert_eq!(r.open_stages(), vec![start]);
        r.phase = Phase::Play;
        r.game.events.push(Event::Cleared);
        r.step(&[Cmd::default(); MAX_PLAYERS]);
        r.phase = Phase::Clear { t: 1 };
        r.step(&[Cmd::default(); MAX_PLAYERS]);
        let Phase::Map { cursor } = r.phase else { panic!("a clear on a map world shows the map, not {:?}", r.phase) };
        assert_eq!(cursor, start + 1);
        assert_eq!(r.open_stages(), vec![start, start + 1, start + 2]);
        r.map_move(crate::game::Dir::Down);
        assert_eq!(r.phase, Phase::Map { cursor: start + 2 }, "down picks the lower way");
        let press = std::array::from_fn(|i| Cmd { bomb: i == 0, ..Cmd::default() });
        r.step(&press);
        assert_eq!(r.index, start + 2);
        assert_eq!(r.phase, Phase::Card(0.0));
    }

    #[test]
    fn payloads_command_is_typed_deleted_and_entered() {
        let start = WORLDS[..9].iter().sum::<usize>();
        let mut r = Run::new(1, 1, start);
        assert!(!r.game.line.is_empty(), "the Prompt has a command line");
        r.phase = Phase::Play;
        r.game.baddies.clear();
        r.game.players[0].invuln = u32::MAX / 2;
        let every = r.stage.typing * HZ / 2;
        for _ in 0..6 * HZ + 3 * every {
            r.step(&[Cmd::default(); MAX_PLAYERS]);
        }
        let (typed, of) = r.game.line_typed();
        assert!(typed >= 2 && typed < of, "it types a character at a time ({typed}/{of})");
        // a blast deletes the first character; typing picks up there again
        let (x, y) = r.game.line[0];
        r.game.tiles[y][x] = crate::game::Tile::Burning(1);
        r.step(&[Cmd::default(); MAX_PLAYERS]);
        assert_eq!(r.game.line_cursor(), Some((x, y)));
        // left alone it finishes, and pressing Enter costs a life
        for _ in 0..(of as u32 + 2) * every {
            r.step(&[Cmd::default(); MAX_PLAYERS]);
            if r.phase != Phase::Play {
                break;
            }
        }
        assert!(r.entered);
        assert_eq!(r.lives[0], LIVES - 1);
    }
}
