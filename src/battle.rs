//! Rounds and matches. With one player it is practice (respawn and play on); a second
//! player joining opens the battle setup, where anyone can pick the stage and the
//! rules and start. Last one standing takes the round, first to `to_win` rounds takes
//! the match. Players who join mid-match come in next round.

use crate::game::{Cmd, Dir, Event, Game, HURRY_AT, HZ, MAX_PLAYERS};
use crate::stage::{self, Stage};

/// The frozen "ready" moment before each round.
pub const READY: u32 = 90;
/// "GO!" stays up this long into the fight.
pub const GO: u32 = 40;
/// After the last knockout the flames finish before the round is called.
const SETTLE: u32 = HZ * 3 / 2;
const SCORES: u32 = HZ * 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Revenge {
    Off,
    On,
    /// A rider who knocks someone out takes their place.
    Comeback,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Options {
    /// A fixed stage, or `None` to play them all in turn.
    pub stage: Option<usize>,
    pub to_win: u32,
    pub revenge: Revenge,
    pub sudden: bool,
    /// Percent of soft blocks hiding an item: 20, 35 or 50.
    pub items: u32,
}

impl Default for Options {
    fn default() -> Options {
        Options { stage: None, to_win: 3, revenge: Revenge::Comeback, sudden: true, items: 35 }
    }
}

/// The setup screen's rows.
pub const ROWS: usize = 5;

impl Options {
    /// The label and the value text of setup row `r`.
    pub fn row(&self, r: usize) -> (&'static str, String) {
        match r {
            0 => ("Stage", self.stage.map_or("all, in turn".into(), |s| format!("{}. {}", s + 1, stage::load(s).name))),
            1 => ("Wins to take the match", self.to_win.to_string()),
            2 => (
                "Knocked out",
                match self.revenge {
                    Revenge::Off => "out for the round".into(),
                    Revenge::On => "ride the border".into(),
                    Revenge::Comeback => "ride, and come back".into(),
                },
            ),
            3 => ("Sudden death", if self.sudden { "after 2:00".into() } else { "off".into() }),
            _ => (
                "Items",
                match self.items {
                    20 => "few".into(),
                    50 => "lots".into(),
                    _ => "normal".into(),
                },
            ),
        }
    }

    fn change(&mut self, r: usize, by: i32) {
        match r {
            0 => {
                // None, then each stage
                let n = stage::count() as i32 + 1;
                let cur = self.stage.map_or(0, |s| s as i32 + 1);
                let next = (cur + by).rem_euclid(n);
                self.stage = if next == 0 { None } else { Some(next as usize - 1) };
            }
            1 => self.to_win = (self.to_win as i32 + by).clamp(1, 5) as u32,
            2 => {
                let all = [Revenge::Off, Revenge::On, Revenge::Comeback];
                let i = all.iter().position(|&v| v == self.revenge).unwrap_or(0) as i32;
                self.revenge = all[(i + by).rem_euclid(3) as usize];
            }
            3 => self.sudden = !self.sudden,
            _ => {
                let all = [20, 35, 50];
                let i = all.iter().position(|&v| v == self.items).unwrap_or(1) as i32;
                self.items = all[(i + by).rem_euclid(3) as usize];
            }
        }
    }
}

/// One player's numbers over a match.
#[derive(Clone, Copy, Default, Debug)]
pub struct Tally {
    pub kills: u32,
    /// Times knocked out.
    pub outs: u32,
    pub blocks: u32,
    pub items: u32,
    pub bombs: u32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Practice,
    /// Battle setup; the cursor is the selected row.
    Lobby(usize),
    Ready(u32),
    Fight,
    /// One or nobody left standing; steps left before the round is called.
    Settle(u32),
    RoundOver { winner: Option<usize>, t: u32 },
    MatchOver { winner: usize },
}

pub struct Match {
    pub game: Game,
    pub stage: Stage,
    /// Which stage this is (0-based).
    pub stage_no: usize,
    pub wins: [u32; MAX_PLAYERS],
    pub tally: [Tally; MAX_PLAYERS],
    pub round: u32,
    pub phase: Phase,
    pub options: Options,
    /// Seats taken (players 0..players).
    pub players: usize,
    seed: u64,
}

impl Match {
    pub fn new(seed: u64, players: usize) -> Match {
        let stage_no = (seed % stage::count() as u64) as usize;
        let stage = stage::load(stage_no);
        let mut m = Match {
            game: Game::from_stage(&stage, seed),
            stage,
            stage_no,
            wins: [0; MAX_PLAYERS],
            tally: [Tally::default(); MAX_PLAYERS],
            round: 0,
            phase: Phase::Practice,
            options: Options::default(),
            players,
            seed,
        };
        if players >= 2 {
            m.to_lobby();
        } else {
            for i in 0..players {
                m.game.join(i);
            }
        }
        m
    }

    fn set_stage(&mut self, n: usize) {
        self.stage_no = n % stage::count();
        self.stage = stage::load(self.stage_no);
        let seed = self.seed ^ (self.round as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        self.game = Game::from_stage(&self.stage, seed);
    }

    fn to_lobby(&mut self) {
        self.phase = Phase::Lobby(0);
        // the arena behind the menu shows the stage you are choosing
        let n = self.options.stage.unwrap_or(self.stage_no);
        self.set_stage(n);
    }

    fn start_match(&mut self) {
        self.wins = [0; MAX_PLAYERS];
        self.tally = [Tally::default(); MAX_PLAYERS];
        self.round = 0;
        self.start_round();
    }

    fn start_round(&mut self) {
        self.round += 1;
        let n = match self.options.stage {
            Some(s) => s,
            None if self.round == 1 => self.stage_no,
            None => self.stage_no + 1,
        };
        self.set_stage(n);
        let o = self.options;
        let g = &mut self.game;
        g.battle = self.players >= 2;
        g.revenge = o.revenge != Revenge::Off;
        g.comeback = o.revenge == Revenge::Comeback;
        g.hurry_at = if o.sudden { HURRY_AT } else { u32::MAX };
        g.item_chance = o.items;
        for i in 0..self.players {
            g.join(i);
        }
        self.phase = if self.game.battle { Phase::Ready(READY) } else { Phase::Practice };
    }

    /// A new player takes seat `players`. The second player opens the battle setup;
    /// later ones come in next round (or join the setup).
    pub fn add_player(&mut self) -> usize {
        let i = self.players;
        self.players += 1;
        if self.players == 2 {
            self.to_lobby();
        } else if self.players == 1 {
            self.game.join(0);
        }
        i
    }

    pub fn battle(&self) -> bool {
        self.players >= 2
    }

    /// Is player `i` waiting to come in next round?
    pub fn waiting(&self, i: usize) -> bool {
        i < self.players && !self.game.players[i].joined && !matches!(self.phase, Phase::Lobby(_))
    }

    /// Advance one step. Returns the game's events.
    pub fn step(&mut self, cmds: &[Cmd; MAX_PLAYERS]) -> Vec<Event> {
        let pressed: Vec<Dir> = cmds.iter().filter_map(|c| c.tap).collect();
        let bomb = cmds.iter().any(|c| c.bomb);
        match self.phase {
            Phase::Practice => self.game.step(cmds),
            Phase::Lobby(row) => {
                let mut row = row;
                for d in pressed {
                    match d {
                        Dir::Up => row = (row + ROWS - 1) % ROWS,
                        Dir::Down => row = (row + 1) % ROWS,
                        Dir::Left | Dir::Right => {
                            let before = self.options.stage;
                            self.options.change(row, if d == Dir::Left { -1 } else { 1 });
                            if row == 0 && self.options.stage != before {
                                let n = self.options.stage.unwrap_or(self.stage_no);
                                self.set_stage(n);
                            }
                        }
                    }
                }
                self.phase = Phase::Lobby(row);
                if bomb {
                    self.start_match();
                }
            }
            Phase::Ready(t) => {
                self.phase = if t <= 1 { Phase::Fight } else { Phase::Ready(t - 1) };
            }
            Phase::Fight => {
                self.game.step(cmds);
                if self.game.standing().len() <= 1 {
                    self.phase = Phase::Settle(SETTLE);
                }
            }
            Phase::Settle(t) => {
                self.game.step(cmds);
                if t > 1 {
                    self.phase = Phase::Settle(t - 1);
                } else {
                    self.end_round();
                }
            }
            Phase::RoundOver { winner, t } => {
                if t <= 1 {
                    self.start_round();
                } else {
                    self.phase = Phase::RoundOver { winner, t: t - 1 };
                }
            }
            Phase::MatchOver { .. } => {
                if bomb {
                    self.to_lobby();
                }
            }
        }
        std::mem::take(&mut self.game.events)
    }

    fn end_round(&mut self) {
        let standing = self.game.standing();
        let winner = (standing.len() == 1).then(|| standing[0]);
        for i in 0..self.players {
            let p = &self.game.players[i];
            if !p.joined {
                continue;
            }
            let t = &mut self.tally[i];
            t.kills += p.kills;
            t.outs += p.deaths;
            t.blocks += p.blocks;
            t.items += p.items_got;
            t.bombs += p.dropped;
        }
        if let Some(w) = winner {
            self.wins[w] += 1;
        }
        self.phase = match winner {
            Some(w) if self.wins[w] >= self.options.to_win => Phase::MatchOver { winner: w },
            _ => Phase::RoundOver { winner, t: SCORES },
        };
    }

    /// End-of-match awards: (title, player) for whoever leads each tally.
    pub fn awards(&self) -> Vec<(&'static str, usize)> {
        let n = self.players;
        let best = |f: &dyn Fn(&Tally) -> i64| -> Option<usize> {
            let top = (0..n).max_by_key(|&i| (f(&self.tally[i]), std::cmp::Reverse(i)))?;
            // only a clear leader with something to show gets it
            let v = f(&self.tally[top]);
            let tied = (0..n).filter(|&i| f(&self.tally[i]) == v).count() > 1;
            (v > 0 && !tied).then_some(top)
        };
        let mut out = Vec::new();
        if let Some(i) = best(&|t| t.kills as i64) {
            out.push(("Sharpshooter: most knockouts", i));
        }
        if let Some(i) = best(&|t| t.blocks as i64) {
            out.push(("Demolition: most blocks broken", i));
        }
        if let Some(i) = best(&|t| t.items as i64) {
            out.push(("Collector: most items", i));
        }
        if let Some(i) = best(&|t| t.bombs as i64) {
            out.push(("Pyromaniac: most bombs dropped", i));
        }
        if let Some(i) = best(&|t| 100 - t.outs as i64) {
            out.push(("Survivor: knocked out least", i));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Flame;

    fn idle() -> [Cmd; MAX_PLAYERS] {
        [Cmd::default(); MAX_PLAYERS]
    }

    fn start(m: &mut Match) {
        let mut c = idle();
        c[0].bomb = true;
        m.step(&c);
        for _ in 0..READY {
            m.step(&idle());
        }
    }

    #[test]
    fn second_player_opens_the_setup() {
        let mut m = Match::new(3, 1);
        assert_eq!(m.phase, Phase::Practice);
        m.add_player();
        assert_eq!(m.phase, Phase::Lobby(0));
        start(&mut m);
        assert_eq!(m.phase, Phase::Fight);
        assert!(m.game.battle);
    }

    #[test]
    fn setup_changes_options() {
        let mut m = Match::new(3, 2);
        let mut c = idle();
        c[1].tap = Some(Dir::Down);
        m.step(&c);
        c[1].tap = Some(Dir::Right);
        m.step(&c);
        assert_eq!(m.phase, Phase::Lobby(1));
        assert_eq!(m.options.to_win, 4);
    }

    #[test]
    fn last_one_standing_takes_the_round() {
        let mut m = Match::new(3, 2);
        m.options.revenge = Revenge::Off;
        start(&mut m);
        let (x, y) = m.game.players[1].tile();
        m.game.flames[y][x] = Some(Flame { left: 5, links: 0, owner: 0, surge: false });
        for _ in 0..(SETTLE + 2) {
            m.step(&idle());
        }
        assert_eq!(m.wins[0], 1);
        assert_eq!(m.tally[0].kills, 1);
        assert!(matches!(m.phase, Phase::RoundOver { winner: Some(0), .. }));
    }
}
