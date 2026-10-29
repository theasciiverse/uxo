//! Baddies: a roster of species written as data (`baddies/*.txt`), each built from a
//! small vocabulary the SB research showed is enough: a way of moving, a trigger and a
//! reaction, plus properties and its own pixel art.
//!
//! Format: `name:`, `about:`, `hp:`, `speed:` (times a player's base walk), `move:`
//! wander | straight | chase | wall (keeps a wall on its right), `sees:` tiles, `on_sight:` chase | charge, `every:` steps
//! between uses of `ability:` hop | roll, `summon:` another species (bosses), `mimic: yes`
//! (looks like a hedge until you come close), `flies: yes` (over soft blocks), `size:` 1
//! or 2 tiles, `points:`, `egg:` hop | dash (it can be tamed and ridden), `colors:` a list
//! of `c=r,g,b`, then `art:` (and an optional `art2:` walk frame): 8x8 pixel rows, 16x16
//! for size 2, `.` transparent.

use crate::game::{Dir, Event, Game, Tile, MAX_PLAYERS, TILE};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Move {
    Wander,
    Straight,
    Chase,
    /// The right-hand rule: always the rightmost way open, so it follows walls round.
    Wall,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sight {
    None,
    Chase,
    Charge,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ability {
    None,
    Hop,
    Roll,
}

/// What a tamed baddie lets you do when you ride it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ride {
    Hop,
    Dash,
}

#[derive(Clone, Debug)]
pub struct Species {
    pub id: String,
    pub name: String,
    pub about: String,
    pub hp: u32,
    /// Units per step.
    pub speed: i32,
    pub moves: Move,
    pub sees: i32,
    pub on_sight: Sight,
    pub every: u32,
    pub ability: Ability,
    pub summon: Option<String>,
    pub mimic: bool,
    pub flies: bool,
    pub size: i32,
    pub points: u32,
    pub egg: Option<Ride>,
    pub colors: Vec<(char, [u8; 3])>,
    pub art: Vec<String>,
    pub art2: Vec<String>,
}

const BASE: i32 = 12;

pub const FILES: [(&str, &str); 13] = [
    ("grub", include_str!("../baddies/grub.txt")),
    ("beetle", include_str!("../baddies/beetle.txt")),
    ("hedgehog", include_str!("../baddies/hedgehog.txt")),
    ("frog", include_str!("../baddies/frog.txt")),
    ("ram", include_str!("../baddies/ram.txt")),
    ("crow", include_str!("../baddies/crow.txt")),
    ("bramblehog", include_str!("../baddies/bramblehog.txt")),
    // World 2, the Clock
    ("tick", include_str!("../baddies/tick.txt")),
    ("bob", include_str!("../baddies/bob.txt")),
    ("spring", include_str!("../baddies/spring.txt")),
    ("cog", include_str!("../baddies/cog.txt")),
    ("alarm", include_str!("../baddies/alarm.txt")),
    ("cuckoo", include_str!("../baddies/cuckoo.txt")),
];

pub fn roster() -> Vec<Species> {
    FILES.iter().map(|(id, text)| parse(id, text).unwrap_or_else(|e| panic!("baddie {id}: {e}"))).collect()
}

pub fn parse(id: &str, text: &str) -> Result<Species, String> {
    let mut s = Species {
        id: id.to_string(),
        name: id.to_string(),
        about: String::new(),
        hp: 1,
        speed: BASE / 2,
        moves: Move::Wander,
        sees: 0,
        on_sight: Sight::None,
        every: 0,
        ability: Ability::None,
        summon: None,
        mimic: false,
        flies: false,
        size: 1,
        points: 100,
        egg: None,
        colors: Vec::new(),
        art: Vec::new(),
        art2: Vec::new(),
    };
    let mut which = 0;
    for line in text.lines() {
        if which > 0 && !line.contains(':') {
            if !line.trim().is_empty() {
                if which == 1 { s.art.push(line.to_string()) } else { s.art2.push(line.to_string()) }
            }
            continue;
        }
        let Some((k, v)) = line.split_once(':') else { continue };
        let v = v.trim();
        let bad = || format!("bad {k}: {v:?}");
        match k.trim() {
            "name" => s.name = v.to_string(),
            "about" => s.about = v.to_string(),
            "hp" => s.hp = v.parse().map_err(|_| bad())?,
            "speed" => s.speed = ((v.parse::<f32>().map_err(|_| bad())? * BASE as f32).round() as i32).max(1),
            "move" => {
                s.moves = match v {
                    "wander" => Move::Wander,
                    "straight" => Move::Straight,
                    "chase" => Move::Chase,
                    "wall" => Move::Wall,
                    _ => return Err(bad()),
                }
            }
            "sees" => s.sees = v.parse().map_err(|_| bad())?,
            "on_sight" => {
                s.on_sight = match v {
                    "chase" => Sight::Chase,
                    "charge" => Sight::Charge,
                    "none" => Sight::None,
                    _ => return Err(bad()),
                }
            }
            "every" => s.every = v.parse().map_err(|_| bad())?,
            "ability" => {
                s.ability = match v {
                    "hop" => Ability::Hop,
                    "roll" => Ability::Roll,
                    "none" => Ability::None,
                    _ => return Err(bad()),
                }
            }
            "summon" => s.summon = Some(v.to_string()),
            "mimic" => s.mimic = v == "yes",
            "flies" => s.flies = v == "yes",
            "size" => s.size = v.parse().map_err(|_| bad())?,
            "points" => s.points = v.parse().map_err(|_| bad())?,
            "egg" => {
                s.egg = match v {
                    "hop" => Some(Ride::Hop),
                    "dash" => Some(Ride::Dash),
                    "none" => None,
                    _ => return Err(bad()),
                }
            }
            "colors" => {
                for part in v.split_whitespace() {
                    let (c, rgb) = part.split_once('=').ok_or(bad())?;
                    let n: Vec<u8> = rgb.split(',').map(|x| x.parse().map_err(|_| bad())).collect::<Result<_, _>>()?;
                    if n.len() != 3 || c.chars().count() != 1 {
                        return Err(bad());
                    }
                    s.colors.push((c.chars().next().unwrap(), [n[0], n[1], n[2]]));
                }
            }
            "art" => which = 1,
            "art2" => which = 2,
            other => return Err(format!("unknown field {other:?}")),
        }
    }
    let px = 8 * s.size as usize;
    for (n, art) in [&s.art, &s.art2].into_iter().enumerate() {
        if n == 1 && art.is_empty() {
            continue;
        }
        if art.len() != px || art.iter().any(|r| r.chars().count() != px) {
            return Err(format!("art{} must be {px}x{px}", if n == 1 { "2" } else { "" }));
        }
        for c in art.iter().flat_map(|r| r.chars()) {
            if c != '.' && !s.colors.iter().any(|(k, _)| *k == c) {
                return Err(format!("art uses {c:?}, which has no colour"));
            }
        }
    }
    Ok(s)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    /// Walking toward the next tile.
    Walk,
    /// Rushing straight ahead until something stops it.
    Charge,
    /// Standing still for a while.
    Pause(u32),
    /// In the air between two tile centres; can't be hurt.
    Hop { from: (i32, i32), to: (i32, i32), t: u32, dur: u32 },
    /// Curled up (can't be hurt), about to roll.
    Curl(u32),
    /// Rolling straight ahead, fast, can't be hurt.
    Roll,
}

#[derive(Clone, Debug)]
pub struct Baddie {
    pub species: usize,
    /// Centre of the top-left tile it stands on, in units.
    pub x: i32,
    pub y: i32,
    pub dir: Dir,
    pub hp: u32,
    pub state: State,
    pub timer: u32,
    /// Steps of flashing after a hit, unhurtable meanwhile.
    pub hurt: u32,
    pub disguised: bool,
    pub alive: bool,
    /// Steps it has walked, for the walk animation.
    pub walk: u32,
}

impl Baddie {
    pub fn new(species: usize, sp: &Species, tx: usize, ty: usize, timer: u32) -> Baddie {
        Baddie {
            species,
            x: centre(tx),
            y: centre(ty),
            dir: Dir::Down,
            hp: sp.hp,
            state: State::Walk,
            timer,
            hurt: 0,
            disguised: sp.mimic,
            alive: true,
            walk: 0,
        }
    }
    pub fn tile(&self) -> (usize, usize) {
        (self.x.div_euclid(TILE).max(0) as usize, self.y.div_euclid(TILE).max(0) as usize)
    }
    /// Can't be hurt right now.
    pub fn immune(&self) -> bool {
        self.hurt > 0 || matches!(self.state, State::Hop { .. } | State::Curl(_) | State::Roll)
    }
    /// The world point at the middle of its body (a size-2 boss is centred on its 2x2).
    pub fn middle(&self, size: i32) -> (i32, i32) {
        (self.x + (size - 1) * TILE / 2, self.y + (size - 1) * TILE / 2)
    }
}

fn centre(t: usize) -> i32 {
    t as i32 * TILE + TILE / 2
}

const DIRS: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

/// A quarter turn to the right (clockwise, seen from above).
fn right_of(d: Dir) -> Dir {
    match d {
        Dir::Up => Dir::Right,
        Dir::Right => Dir::Down,
        Dir::Down => Dir::Left,
        Dir::Left => Dir::Up,
    }
}

fn back(d: Dir) -> Dir {
    match d {
        Dir::Up => Dir::Down,
        Dir::Down => Dir::Up,
        Dir::Left => Dir::Right,
        Dir::Right => Dir::Left,
    }
}

impl Game {
    /// Can a baddie of species `sp` (size and all) stand with its top-left on (x, y)?
    fn baddie_free(&self, sp: &Species, x: i32, y: i32) -> bool {
        for dy in 0..sp.size {
            for dx in 0..sp.size {
                let (tx, ty) = (x + dx, y + dy);
                if tx < 1 || ty < 1 || tx >= self.w as i32 - 1 || ty >= self.h as i32 - 1 {
                    return false;
                }
                let (ux, uy) = (tx as usize, ty as usize);
                let ok = match self.tiles[uy][ux] {
                    Tile::Floor => true,
                    Tile::Soft => sp.flies,
                    _ => false,
                };
                if !ok || self.bomb_at(ux, uy).is_some() || self.exit == Some((ux, uy)) {
                    return false;
                }
            }
        }
        true
    }

    /// The player in line with this baddie within `range` tiles, with nothing solid
    /// between, and which way they are.
    fn sighted(&self, b: &Baddie, range: i32) -> Option<Dir> {
        let (bx, by) = b.tile();
        for p in self.players.iter().filter(|p| p.joined && p.alive) {
            let (px, py) = p.tile();
            let (dx, dy) = (px as i32 - bx as i32, py as i32 - by as i32);
            if (dx != 0 && dy != 0) || (dx == 0 && dy == 0) || dx.abs().max(dy.abs()) > range {
                continue;
            }
            let (sx, sy) = (dx.signum(), dy.signum());
            let clear = (1..dx.abs().max(dy.abs())).all(|k| {
                let (x, y) = ((bx as i32 + sx * k) as usize, (by as i32 + sy * k) as usize);
                self.tiles[y][x] == Tile::Floor && self.bomb_at(x, y).is_none()
            });
            if clear {
                return Some(match (sx, sy) {
                    (1, _) => Dir::Right,
                    (-1, _) => Dir::Left,
                    (_, 1) => Dir::Down,
                    _ => Dir::Up,
                });
            }
        }
        None
    }

    /// The nearest player still in play, for chasers.
    fn nearest_player(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        self.players
            .iter()
            .filter(|p| p.joined && p.alive)
            .map(|p| (p.x, p.y))
            .min_by_key(|&(px, py)| (px - x).abs() + (py - y).abs())
    }

    pub fn step_baddies(&mut self) {
        for k in 0..self.baddies.len() {
            if !self.baddies[k].alive {
                continue;
            }
            self.step_baddie(k);
        }
        self.baddies_hurt();
        self.baddies.retain(|b| b.alive);
    }

    fn step_baddie(&mut self, k: usize) {
        let sp = self.species[self.baddies[k].species].clone();
        let b = &mut self.baddies[k];
        b.hurt = b.hurt.saturating_sub(1);
        b.timer = b.timer.saturating_sub(1);

        if b.disguised {
            // wakes when someone comes near, or a blast goes off beside it
            let (bx, by) = b.tile();
            let near = self.players.iter().any(|p| p.joined && p.alive && p.tile().0.abs_diff(bx) + p.tile().1.abs_diff(by) <= 2);
            let blast = DIRS.iter().any(|d| {
                let (dx, dy) = d.vec();
                self.flames[(by as i32 + dy) as usize][(bx as i32 + dx) as usize].is_some()
            });
            if near || blast {
                let b = &mut self.baddies[k];
                b.disguised = false;
                b.state = State::Pause(30);
            }
            return;
        }

        match self.baddies[k].state {
            State::Pause(t) => {
                self.baddies[k].state = if t <= 1 { State::Walk } else { State::Pause(t - 1) };
                return;
            }
            State::Hop { from, to, t, dur } => {
                let t = t + 1;
                let b = &mut self.baddies[k];
                if t >= dur {
                    (b.x, b.y) = to;
                    b.state = State::Pause(20);
                } else {
                    b.x = from.0 + (to.0 - from.0) * t as i32 / dur as i32;
                    b.y = from.1 + (to.1 - from.1) * t as i32 / dur as i32;
                    b.state = State::Hop { from, to, t, dur };
                }
                return;
            }
            State::Curl(t) => {
                if t <= 1 {
                    // roll toward the nearest player, along whichever axis is longer
                    let (x, y) = (self.baddies[k].x, self.baddies[k].y);
                    if let Some((px, py)) = self.nearest_player(x, y) {
                        let (dx, dy) = (px - x, py - y);
                        self.baddies[k].dir = if dx.abs() > dy.abs() {
                            if dx < 0 { Dir::Left } else { Dir::Right }
                        } else if dy < 0 {
                            Dir::Up
                        } else {
                            Dir::Down
                        };
                    }
                    self.baddies[k].state = State::Roll;
                } else {
                    self.baddies[k].state = State::Curl(t - 1);
                }
                return;
            }
            _ => {}
        }

        let b = &self.baddies[k];
        let (tx, ty) = b.tile();
        let at_centre = b.x == centre(tx) && b.y == centre(ty);
        let fast = matches!(b.state, State::Charge | State::Roll);
        let speed = if fast { sp.speed * 4 } else { sp.speed };

        if at_centre {
            let (x, y) = (tx as i32, ty as i32);
            // abilities on a timer
            if sp.every > 0 && b.timer == 0 && !fast {
                self.baddies[k].timer = sp.every + self.rand_u(sp.every / 2 + 1);
                // calling up help comes first: the ability below may start a move and return
                if let Some(id) = &sp.summon {
                    self.summon(id, tx, ty);
                }
                match sp.ability {
                    Ability::Hop => {
                        let d = self.baddies[k].dir;
                        let (dx, dy) = d.vec();
                        // over anything but the outer wall, onto free floor
                        if self.baddie_free(&sp, x + dx * 2, y + dy * 2) {
                            let to = (centre((x + dx * 2) as usize), centre((y + dy * 2) as usize));
                            self.baddies[k].state = State::Hop { from: (self.baddies[k].x, self.baddies[k].y), to, t: 0, dur: 30 };
                            return;
                        }
                    }
                    Ability::Roll => {
                        self.baddies[k].state = State::Curl(40);
                        return;
                    }
                    Ability::None => {}
                }
            }
            let b = &self.baddies[k];
            let mut dir = b.dir;
            let mut state = b.state;
            if fast {
                // charging and rolling go on until blocked
                let (dx, dy) = dir.vec();
                if !self.baddie_free(&sp, x + dx, y + dy) {
                    state = State::Pause(if state == State::Roll { 40 } else { 30 });
                    self.baddies[k].state = state;
                    return;
                }
            } else {
                let seen = if sp.sees > 0 { self.sighted(b, sp.sees) } else { None };
                if let (Some(d), Sight::Charge) = (seen, sp.on_sight) {
                    dir = d;
                    state = State::Charge;
                } else {
                    let want = match (seen, sp.on_sight, sp.moves) {
                        (Some(d), Sight::Chase, _) => Some(d),
                        (_, _, Move::Chase) => self.nearest_player(b.x, b.y).map(|(px, py)| {
                            let (dx, dy) = (px - b.x, py - b.y);
                            if dx.abs() > dy.abs() {
                                if dx < 0 { Dir::Left } else { Dir::Right }
                            } else if dy < 0 {
                                Dir::Up
                            } else {
                                Dir::Down
                            }
                        }),
                        _ => None,
                    };
                    dir = self.choose_dir(&sp, x, y, dir, want);
                    let (dx, dy) = dir.vec();
                    if !self.baddie_free(&sp, x + dx, y + dy) {
                        self.baddies[k].state = State::Pause(15);
                        return;
                    }
                }
            }
            let b = &mut self.baddies[k];
            b.dir = dir;
            b.state = state;
        } else {
            // mid-way: if a bomb landed in the way, turn back
            let (dx, dy) = b.dir.vec();
            let ahead_x = (b.x + dx * (TILE / 2)).div_euclid(TILE);
            let ahead_y = (b.y + dy * (TILE / 2)).div_euclid(TILE);
            let past = ((b.x - centre(tx)) * dx + (b.y - centre(ty)) * dy) > 0;
            if past && !fast && !self.baddie_free(&sp, ahead_x, ahead_y) && (ahead_x, ahead_y) != (tx as i32, ty as i32) {
                let b = &mut self.baddies[k];
                b.dir = back(b.dir);
            }
        }

        // move toward the next tile centre, never past it
        let b = &mut self.baddies[k];
        let (dx, dy) = b.dir.vec();
        let (tx, ty) = b.tile();
        let (cx, cy) = (centre(tx), centre(ty));
        let to_centre = (cx - b.x) * dx + (cy - b.y) * dy;
        let step = if to_centre > 0 { speed.min(to_centre) } else { speed };
        b.x += dx * step;
        b.y += dy * step;
        // keep to the lane on the other axis
        if dx != 0 {
            b.y = cy;
        } else {
            b.x = cx;
        }
        b.walk += 1;
        // a rolling boss flattens the soft blocks in its way
        if b.state == State::Roll {
            let (nx, ny) = b.tile();
            for oy in 0..sp.size as usize {
                for ox in 0..sp.size as usize {
                    if let Some(row) = self.tiles.get_mut(ny + oy)
                        && let Some(t) = row.get_mut(nx + ox)
                        && *t == Tile::Soft
                    {
                        *t = Tile::Burning(crate::game::BURN);
                    }
                }
            }
        }
    }

    fn rand_u(&mut self, n: u32) -> u32 {
        self.rand_pub(n.max(1))
    }

    /// Pick the next direction at a tile centre.
    fn choose_dir(&mut self, sp: &Species, x: i32, y: i32, cur: Dir, want: Option<Dir>) -> Dir {
        let free = |g: &Game, d: Dir| {
            let (dx, dy) = d.vec();
            g.baddie_free(sp, x + dx, y + dy)
        };
        if let Some(d) = want
            && free(self, d)
        {
            return d;
        }
        let open: Vec<Dir> = DIRS.iter().copied().filter(|&d| free(self, d)).collect();
        if open.is_empty() {
            return cur;
        }
        match sp.moves {
            Move::Wall => {
                // a wall on the right, or one just passed (round its corner, then)
                let right = right_of(cur);
                let ((rx, ry), (bx, by)) = (right.vec(), back(cur).vec());
                let walled = !free(self, right) || !self.baddie_free(sp, x + rx + bx, y + ry + by);
                // with no wall to follow, straight on until one turns it left
                let order = if walled { [right, cur, back(right), back(cur)] } else { [cur, back(right), right, back(cur)] };
                order.into_iter().find(|&d| free(self, d)).unwrap_or(cur)
            }
            Move::Straight if free(self, cur) => cur,
            Move::Wander if free(self, cur) && self.rand_u(100) < 70 => cur,
            _ => {
                // anything but straight back, unless that's the only way
                let fwd: Vec<Dir> = open.iter().copied().filter(|&d| d != back(cur)).collect();
                let pool = if fwd.is_empty() { &open } else { &fwd };
                pool[self.rand_u(pool.len() as u32) as usize]
            }
        }
    }

    fn summon(&mut self, id: &str, bx: usize, by: usize) {
        let Some(si) = self.species.iter().position(|s| s.id == id) else { return };
        if self.baddies.iter().filter(|b| b.alive && b.species == si).count() >= 4 {
            return;
        }
        let sp = self.species[si].clone();
        // a free tile near the summoner
        for r in 2..6i32 {
            for (dx, dy) in [(r, 0), (-r, 0), (0, r), (0, -r)] {
                let (x, y) = (bx as i32 + dx, by as i32 + dy);
                if self.baddie_free(&sp, x, y) && !self.players.iter().any(|p| p.alive && p.tile() == (x as usize, y as usize)) {
                    let t = self.rand_u(60);
                    self.baddies.push(Baddie::new(si, &sp, x as usize, y as usize, t));
                    return;
                }
            }
        }
    }

    /// Blasts hurt baddies; baddies hurt players they touch.
    fn baddies_hurt(&mut self) {
        for k in 0..self.baddies.len() {
            let b = &self.baddies[k];
            if !b.alive {
                continue;
            }
            let sp = &self.species[b.species];
            let size = sp.size;
            let (tx, ty) = b.tile();
            // flames on any tile of its body
            let mut by = None;
            for oy in 0..size as usize {
                for ox in 0..size as usize {
                    if let Some(f) = self.flames.get(ty + oy).and_then(|r| r.get(tx + ox)).copied().flatten() {
                        by = Some(f.owner);
                    }
                }
            }
            if let Some(owner) = by
                && !b.immune()
            {
                let b = &mut self.baddies[k];
                b.disguised = false;
                b.hp = b.hp.saturating_sub(1);
                b.hurt = 40;
                if b.hp == 0 {
                    b.alive = false;
                    let (species, (mx, my)) = (b.species, b.middle(size));
                    let home = crate::game::Homing { x: b.x, y: b.y, species, flip: b.dir == Dir::Left, left: crate::game::HOMING };
                    self.events.push(Event::BaddieDown { species, by: owner, x: mx, y: my });
                    self.homing.push(home);
                    if self.species[species].egg.is_some() {
                        self.eggs.push(crate::game::Egg { x: centre(tx), y: centre(ty), species });
                    }
                } else {
                    self.events.push(Event::BaddieHit { species: b.species });
                }
                continue;
            }
            if b.disguised || matches!(b.state, State::Hop { .. }) {
                continue;
            }
            // touching a player hurts them
            let (mx, my) = b.middle(size);
            let reach = TILE * (size + 1) / 2 - TILE / 4;
            for i in 0..MAX_PLAYERS {
                let p = &self.players[i];
                if p.joined && p.alive && (p.x - mx).abs() < reach && (p.y - my).abs() < reach {
                    self.hurt_pub(i, None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Cmd, Flame, Goal};

    #[test]
    fn roster_parses() {
        let r = roster();
        assert!(r.len() >= 7);
        assert!(r.iter().any(|s| s.size == 2));
    }

    /// A little test stage: `extra` header lines, then the grid.
    fn stage(extra: &str, grid: &str) -> Game {
        let text = format!(
            "name: Test\nworld: Test\n{extra}baddie: g grub\nbaddie: f frog\nbaddie: r ram\nbaddie: h hedgehog\ngrid:\n{grid}"
        );
        let st = crate::stage::parse(&text).unwrap();
        let mut g = Game::from_stage(&st, 5);
        g.adventure = true;
        g.join(0);
        g.join(1);
        g
    }

    const OPEN: &str = "#########\n#1.2....#\n#.......#\n#...g...#\n#.......#\n#......E#\n#########";

    fn idle() -> [Cmd; MAX_PLAYERS] {
        [Cmd::default(); MAX_PLAYERS]
    }

    #[test]
    fn a_blast_beats_a_baddie_and_the_hatch_opens() {
        let mut g = stage("goal: clear\n", OPEN);
        assert!(!g.exit_open);
        let (x, y) = g.baddies[0].tile();
        g.flames[y][x] = Some(Flame { left: 5, links: 0, owner: 0, surge: false });
        g.step(&idle());
        assert!(g.baddies.is_empty());
        assert!(g.exit_open, "beating every baddie opens the hatch");
        // walk onto the hatch: the stage is cleared
        g.players[0].x = 7 * TILE + TILE / 2;
        g.players[0].y = 5 * TILE + TILE / 2;
        g.step(&idle());
        assert!(g.cleared);
    }

    #[test]
    fn a_beaten_baddie_flickers_home_and_is_gone() {
        let mut g = stage("goal: clear\n", OPEN);
        let (x, y) = g.baddies[0].tile();
        g.flames[y][x] = Some(Flame { left: 5, links: 0, owner: 0, surge: false });
        g.step(&idle());
        assert_eq!(g.homing.len(), 1, "the kill sends it home");
        for _ in 0..crate::game::HOMING {
            g.step(&idle());
        }
        assert!(g.homing.is_empty());
    }

    #[test]
    fn a_tick_follows_the_walls_round() {
        let grid = "###########\n#1.2......#\n#.........#\n#....t....#\n#.........#\n#........E#\n###########";
        let mut g = stage("goal: clear\nbaddie: t tick\n", grid);
        let mut seen = std::collections::HashSet::new();
        for n in 0..2400 {
            g.step(&idle());
            let b = &g.baddies[0];
            let (x, y) = b.tile();
            if n > 600 && b.x == centre(x) && b.y == centre(y) {
                // once it has found a wall it keeps one beside it (or round the corner
                // of one, the hatch included: baddies can't step on it)
                let by_wall = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))).any(|(dx, dy)| {
                    let (nx, ny) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
                    g.tiles[ny][nx] == Tile::Wall || g.exit == Some((nx, ny))
                });
                assert!(by_wall, "the tick left the wall at {x},{y}");
                seen.insert((x, y));
            }
        }
        assert!(seen.len() >= 16, "it goes round, not back and forth ({} tiles)", seen.len());
    }

    #[test]
    fn the_bramblehog_calls_up_grubs() {
        let grid = "###########\n#1.2......#\n#.........#\n#....x....#\n#.........#\n#.........#\n#........E#\n###########";
        let mut g = stage("goal: clear\nbaddie: x bramblehog\n", grid);
        g.baddies[0].timer = 0;
        for _ in 0..60 {
            g.step(&idle());
        }
        assert!(g.baddies.iter().any(|b| g.species[b.species].id == "grub"));
    }

    #[test]
    fn a_tamed_species_leaves_an_egg_you_can_ride() {
        let mut g = stage("goal: clear\n", &OPEN.replace('g', "f"));
        let (x, y) = g.baddies[0].tile();
        g.flames[y][x] = Some(Flame { left: 5, links: 0, owner: 0, surge: false });
        g.step(&idle());
        assert_eq!(g.eggs.len(), 1);
        // let the flame die down first
        for _ in 0..8 {
            g.step(&idle());
        }
        g.players[0].x = x as i32 * TILE + TILE / 2;
        g.players[0].y = y as i32 * TILE + TILE / 2;
        for _ in 0..3 {
            g.step(&idle());
        }
        assert!(g.players[0].mount.is_some(), "walking over the egg mounts you");
        // the mount takes a hit for you
        g.players[0].invuln = 0;
        g.hurt_pub(0, None);
        assert!(g.players[0].alive);
        assert!(g.players[0].mount.is_none());
    }

    #[test]
    fn touching_a_baddie_hurts() {
        let mut g = stage("goal: clear\n", OPEN);
        let (bx, by) = (g.baddies[0].x, g.baddies[0].y);
        g.baddies[0].state = State::Pause(100);
        g.players[0].x = bx;
        g.players[0].y = by;
        g.step(&idle());
        assert!(!g.players[0].alive);
    }

    #[test]
    fn hopping_mount_clears_a_block() {
        let mut g = stage("goal: clear\n", "#########\n#1.2....#\n#.......#\n#.+.....#\n#.......#\n#..g...E#\n#########");
        let frog = g.species.iter().position(|s| s.id == "frog").unwrap();
        g.players[0].mount = Some(frog);
        g.players[0].facing = crate::game::Dir::Down;
        g.players[0].x = TILE + TILE / 2;
        g.players[0].y = 2 * TILE + TILE / 2;
        g.tiles[3][1] = Tile::Soft;
        let mut c = idle();
        c[0].ride = true;
        g.step(&c);
        for _ in 0..30 {
            g.step(&idle());
        }
        assert_eq!(g.players[0].tile(), (1, 4), "hopped over the hedge");
    }

    #[test]
    fn last_dud_opens_the_hatch() {
        let mut g = stage("goal: duds\n", "#########\n#1.2....#\n#.......#\n#...*...#\n#.......#\n#..g...E#\n#########");
        assert_eq!(g.goal, Goal::Duds);
        g.place_bomb(0, 4, 2);
        g.bombs[0].fuse = 1;
        g.step(&idle());
        assert_eq!(g.tiles[3][4], Tile::Floor);
        assert!(g.exit_open);
        assert!(!g.sparks.is_empty(), "a dud goes up in fireworks");
    }

    #[test]
    fn a_ram_charges_when_it_sees_you() {
        let mut g = stage("goal: clear\n", "###########\n#1.2......#\n#.........#\n#.........#\n#r........#\n#........E#\n###########");
        g.players[0].x = 8 * TILE + TILE / 2;
        g.players[0].y = 4 * TILE + TILE / 2;
        g.players[0].invuln = 9999;
        let mut charged = false;
        for _ in 0..40 {
            g.step(&idle());
            charged |= g.baddies[0].state == State::Charge;
        }
        assert!(charged, "the ram should charge along the row");
    }

    #[test]
    fn a_hedgehog_hides_until_you_come_close() {
        let mut g = stage("goal: clear\n", "###########\n#1.2......#\n#.........#\n#.........#\n#.......h.#\n#........E#\n###########");
        for _ in 0..30 {
            g.step(&idle());
        }
        assert!(g.baddies[0].disguised);
        g.players[0].x = 7 * TILE + TILE / 2;
        g.players[0].y = 3 * TILE + TILE / 2;
        g.players[0].invuln = 9999;
        g.step(&idle());
        assert!(!g.baddies[0].disguised);
    }
}
