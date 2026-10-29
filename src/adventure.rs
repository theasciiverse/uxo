//! Adventure: the worlds, stage by stage, alone or two together. A story card before
//! each stage; lives and score; a clock per stage; the exit opens when the stage's goal
//! is met (beat every baddie, or set off every dud), or the boss falls. Power-ups and
//! mounts carry on to the next stage; a lost life costs the handling items.

use crate::game::{Cmd, Event, Game, HZ, MAX_PLAYERS};
use crate::stage::{self, Stage};

pub const STAGES: [&str; 9] = [
    include_str!("../adventure/orchard/01-windfall-lane.txt"),
    include_str!("../adventure/orchard/02-beetle-rows.txt"),
    include_str!("../adventure/orchard/03-hedge-maze.txt"),
    include_str!("../adventure/orchard/04-frog-pond.txt"),
    include_str!("../adventure/orchard/05-cider-press.txt"),
    include_str!("../adventure/orchard/06-ram-pasture.txt"),
    include_str!("../adventure/orchard/07-crow-tree.txt"),
    include_str!("../adventure/orchard/08-the-old-grove.txt"),
    include_str!("../adventure/orchard/09-bramblehog.txt"),
];

pub fn load(i: usize) -> Stage {
    stage::parse(STAGES[i]).unwrap_or_else(|e| panic!("adventure stage {}: {e}", i + 1))
}

pub const LIVES: u32 = 3;
const DUD_POINTS: u32 = 500;
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
    /// Nothing more yet.
    Done,
}

/// What a stage earned, for the clear screen.
#[derive(Clone, Copy, Default, Debug)]
pub struct Tally {
    pub baddies: u32,
    pub duds: u32,
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
            seed,
        };
        r.begin(false);
        r
    }

    /// "1-3": world and stage number.
    pub fn code(&self) -> String {
        format!("1-{}", self.index + 1)
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
        self.phase = Phase::Card(0.0);
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
            Event::DudPopped { by, .. } => {
                if by < MAX_PLAYERS {
                    self.score[by] += DUD_POINTS;
                }
                self.tally.duds += 1;
            }
            Event::Death { player, .. } if self.phase == Phase::Play => {
                self.lose_life(player);
                if self.all_out() {
                    self.phase = Phase::GameOver;
                }
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
                crate::progress::reached(self.index + 1);
                self.phase = Phase::Clear { t: CLEAR_HOLD };
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
