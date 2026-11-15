//! A computer player for playtesting the adventure (`--bot-test`, `UXO_BOT=1`).
//!
//! It plays player one with foresight: each plan it thinks of (walk somewhere, drop a
//! bomb, walk out of the blast) is first tried on a copy of the game, and only a plan
//! that keeps Dot alive is played. The game is deterministic, so a plan that worked on
//! the copy works for real. It proves stages can be cleared and gives clear times and
//! the places where it got caught; it is no measure of how hard a stage feels.

use crate::adventure::{self, Phase, Run};
use crate::game::{centre, Cmd, Dir, Event, Game, Goal, Ground, Item, Tile, FLAME, FUSE, MAX_PLAYERS, TILE};
use std::collections::VecDeque;

type At = (usize, usize);

#[derive(Clone, Debug)]
enum Act {
    Go(Vec<At>),
    Bomb,
    /// The action button: sets off a remote bomb.
    Fire,
    Wait(u32),
}

#[derive(Clone, Debug, Default)]
struct Plan {
    acts: Vec<Act>,
    k: usize,
    at: usize,
    still: u32,
    last: (i32, i32),
    failed: bool,
    what: &'static str,
    /// Steps played, and the most it may take before it is given up.
    steps: u32,
    limit: u32,
}

impl Plan {
    fn new(acts: Vec<Act>, what: &'static str) -> Plan {
        let mut p = Plan { acts, at: 1, what, ..Plan::default() };
        p.limit = p.length(12) * 2 + 60;
        p
    }

    /// The command for this step, or None when the plan is played out (or stuck).
    fn cmd(&mut self, g: &Game, i: usize) -> Option<Cmd> {
        let p = &g.players[i];
        // mid-hop on a mount: carried, nothing to do
        if p.hop.is_some() {
            return Some(Cmd::default());
        }
        self.steps += 1;
        if self.steps > self.limit {
            self.failed = true;
        }
        loop {
            if self.failed {
                return None;
            }
            let act = self.acts.get(self.k)?;
            match act {
                Act::Go(path) => {
                    if self.at >= path.len() {
                        self.k += 1;
                        self.at = 1;
                        continue;
                    }
                    // carried ahead (a warp, a belt): pick the path up from there
                    let here = p.tile();
                    if let Some(j) = path.iter().skip(self.at + 1).position(|&t| t == here) {
                        self.at += j + 1;
                    }
                    let (tx, ty) = path[self.at];
                    let (dx, dy) = (centre(tx) - p.x, centre(ty) - p.y);
                    let last = self.at + 1 == path.len();
                    let near = if last { TILE / 10 } else { TILE / 4 };
                    if dx.abs() <= near && dy.abs() <= near {
                        self.at += 1;
                        self.still = 0;
                        continue;
                    }
                    if (p.x, p.y) == self.last {
                        self.still += 1;
                        if self.still > 45 {
                            self.failed = true;
                            return None;
                        }
                    } else {
                        self.still = 0;
                    }
                    self.last = (p.x, p.y);
                    let hor = if dx > 0 { Dir::Right } else { Dir::Left };
                    let ver = if dy > 0 { Dir::Down } else { Dir::Up };
                    let (primary, secondary) = if dx.abs() >= dy.abs() {
                        (hor, (dy.abs() > 2).then_some(ver))
                    } else {
                        (ver, (dx.abs() > 2).then_some(hor))
                    };
                    return Some(Cmd { primary: Some(primary), secondary, ..Cmd::default() });
                }
                Act::Bomb => {
                    self.k += 1;
                    return Some(Cmd { bomb: true, ..Cmd::default() });
                }
                Act::Fire => {
                    self.k += 1;
                    return Some(Cmd { action: true, ..Cmd::default() });
                }
                Act::Wait(n) => {
                    let n = *n;
                    if n == 0 {
                        self.k += 1;
                        continue;
                    }
                    self.acts[self.k] = Act::Wait(n - 1);
                    return Some(Cmd::default());
                }
            }
        }
    }

    /// Roughly how many steps it takes.
    fn length(&self, speed: i32) -> u32 {
        self.acts
            .iter()
            .map(|a| match a {
                Act::Go(p) => (p.len() as i32 * TILE / speed.max(1)) as u32 + 12,
                Act::Wait(n) => *n,
                _ => 1,
            })
            .sum()
    }

    fn bombs(&self) -> bool {
        self.acts.iter().any(|a| matches!(a, Act::Bomb))
    }
}

/// What a plan did on the copy of the game.
struct Outcome {
    safe: bool,
    steps: u32,
    gain: i32,
    cleared: bool,
}

fn worth(item: Item) -> i32 {
    match item {
        Item::BombUp | Item::FireUp | Item::SpeedUp | Item::Heart | Item::Vest => 6,
        Item::FullFire | Item::Pierce | Item::BombPass => 4,
        Item::Kick | Item::Punch | Item::Glove | Item::LineBomb | Item::BlockPass | Item::Remote => 0,
        Item::Rubber => -2,
        Item::Surge | Item::Mine | Item::SpeedDown | Item::Skull => -30,
    }
}

/// Try `plan` on a copy of the game for `horizon` steps (standing still once it is done).
fn simulate(g: &Game, plan: &Plan, i: usize, horizon: u32, mut curve: Option<&mut Vec<i32>>) -> Outcome {
    let mut g = g.clone();
    let mut plan = plan.clone();
    let mut done_at = None;
    let mut gain = 0;
    let blocks = g.players[i].blocks;
    let start_y = g.players[i].tile().1;
    for s in 0..horizon {
        let mut cmds = [Cmd::default(); MAX_PLAYERS];
        match plan.cmd(&g, i) {
            Some(c) => cmds[i] = c,
            None => {
                if done_at.is_none() {
                    done_at = Some(s);
                    if plan.failed {
                        gain -= 3;
                    }
                }
            }
        }
        g.step(&cmds);
        for ev in std::mem::take(&mut g.events) {
            match ev {
                Event::Death { player, .. } | Event::Saved { player } if player == i => {
                    return Outcome { safe: false, steps: s, gain, cleared: false };
                }
                Event::Cleared => {
                    return Outcome { safe: true, steps: s, gain: gain + 1000, cleared: true };
                }
                Event::BaddieDown { by, .. } if by == i => gain += 12,
                Event::BaddieHit { .. } => gain += 5,
                Event::DudPopped { .. } => gain += 12,
                Event::Got { player, .. } if player == i => gain += 10,
                Event::Pickup { player, item } if player == i => gain += worth(item),
                Event::Emptied { by, .. } if by == i => gain += 6,
                Event::ExitOpen | Event::SocketOpen => gain += 20,
                Event::DoorOpened { .. } => gain += 6,
                Event::Unpacked { .. } => gain += 2,
                Event::Gates { open: true } => gain += 3,
                _ => {}
            }
        }
        if !g.players[i].alive {
            return Outcome { safe: false, steps: s, gain, cleared: false };
        }
        if let Some(c) = curve.as_deref_mut() {
            c.push(gain + (g.players[i].blocks - blocks) as i32 * 2);
        }
    }
    gain += (g.players[i].blocks - blocks) as i32 * 2;
    // Scrollback: down is the way, and the top is where you get shut in
    if g.scroll_every > 0 {
        let (y0, y1) = (start_y as i32, g.players[i].tile().1 as i32);
        gain += (y1 - y0) * 4;
        if y1 <= g.scroll_top as i32 + 2 {
            gain -= 15;
        }
    }
    Outcome { safe: true, steps: done_at.unwrap_or(horizon), gain, cleared: false }
}

/// Tiles a blast from (x, y) would reach, as a cross; `pierce` goes through soft blocks.
fn cross(g: &Game, x: usize, y: usize, fire: u32, pierce: bool) -> Vec<At> {
    let mut v = vec![(x, y)];
    for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
        for k in 1..=fire as i32 {
            let (nx, ny) = (x as i32 + dx * k, y as i32 + dy * k);
            if nx < 0 || ny < 0 || nx >= g.w as i32 || ny >= g.h as i32 {
                break;
            }
            let t = (nx as usize, ny as usize);
            match g.tiles[t.1][t.0] {
                Tile::Floor => {
                    v.push(t);
                    if g.bomb_at(t.0, t.1).is_some() {
                        break;
                    }
                }
                Tile::Soft | Tile::Burning(_) => {
                    v.push(t);
                    if !pierce {
                        break;
                    }
                }
                Tile::Dud | Tile::Packed => {
                    v.push(t);
                    break;
                }
                _ => break,
            }
        }
    }
    v
}

/// Every tile some bomb on the board would reach, or that is burning now.
fn danger(g: &Game) -> Vec<Vec<bool>> {
    let mut d = vec![vec![false; g.w]; g.h];
    for b in &g.bombs {
        let (x, y) = b.tile();
        if x < g.w && y < g.h {
            for (cx, cy) in cross(g, x, y, b.fire.max(2), false) {
                d[cy][cx] = true;
            }
        }
    }
    for y in 0..g.h {
        for x in 0..g.w {
            if g.flames[y][x].is_some() {
                d[y][x] = true;
            }
        }
    }
    d
}

/// Can player `i` walk onto (x, y) as the board stands (doors open with a key, blocks
/// are pushed out of the way when there is room behind them).
fn open(g: &Game, i: usize, from: At, x: usize, y: usize) -> bool {
    let p = &g.players[i];
    let ground_ok = match g.tiles[y][x] {
        Tile::Floor => true,
        Tile::Soft => p.block_pass,
        Tile::Door(c) => p.keys[c as usize] > 0,
        Tile::Block => {
            let (bx, by) = (2 * x as i32 - from.0 as i32, 2 * y as i32 - from.1 as i32);
            bx > 0
                && by > 0
                && (bx as usize) < g.w
                && (by as usize) < g.h
                && (g.tiles[by as usize][bx as usize] == Tile::Floor
                    || g.ground[by as usize][bx as usize] == Ground::Water)
                && g.bomb_at(bx as usize, by as usize).is_none()
        }
        _ => false,
    };
    ground_ok && (p.bomb_pass || g.bomb_at(x, y).is_none())
}

/// Breadth-first paths from `start`: the parent of every tile reached.
fn paths(g: &Game, i: usize, start: At, extra_bomb: Option<At>) -> Vec<Vec<Option<At>>> {
    // stepping onto a warp sends you on to the next one in reading order
    let warps: Vec<At> = (0..g.h)
        .flat_map(|y| (0..g.w).map(move |x| (x, y)))
        .filter(|&(x, y)| g.ground[y][x] == Ground::Warp && g.tiles[y][x] == Tile::Floor)
        .collect();
    let mut par = vec![vec![None; g.w]; g.h];
    par[start.1][start.0] = Some(start);
    let mut q = VecDeque::from([start]);
    while let Some((x, y)) = q.pop_front() {
        for (dx, dy) in [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if nx < 0 || ny < 0 || nx >= g.w as i32 || ny >= g.h as i32 {
                continue;
            }
            let n = (nx as usize, ny as usize);
            if par[n.1][n.0].is_some() || Some(n) == extra_bomb || !open(g, i, (x, y), n.0, n.1) {
                continue;
            }
            par[n.1][n.0] = Some((x, y));
            if warps.len() > 1
                && let Some(k) = warps.iter().position(|&w| w == n)
            {
                let d = warps[(k + 1) % warps.len()];
                if par[d.1][d.0].is_none() {
                    par[d.1][d.0] = Some(n);
                    q.push_back(d);
                }
                continue;
            }
            q.push_back(n);
        }
    }
    par
}

fn path_to(par: &[Vec<Option<At>>], start: At, to: At) -> Option<Vec<At>> {
    par[to.1][to.0]?;
    let mut v = vec![to];
    let mut at = to;
    while at != start {
        at = par[at.1][at.0]?;
        v.push(at);
    }
    v.reverse();
    Some(v)
}

fn dist(a: At, b: At) -> usize {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}

/// How much a bomb dropped on (x, y) would do, judged by the board alone.
fn bomb_value(g: &Game, i: usize, x: usize, y: usize) -> i32 {
    let p = &g.players[i];
    let pierce = p.kind == crate::game::BombKind::Pierce;
    let mut v = 0;
    for (cx, cy) in cross(g, x, y, p.fire, pierce) {
        match g.tiles[cy][cx] {
            Tile::Soft => v += if g.ground[cy][cx] == Ground::Line { 4 } else { 1 },
            Tile::Packed => v += 1,
            Tile::Dud => v += 8,
            _ => {}
        }
        for b in g.baddies.iter().filter(|b| b.alive) {
            let n = g.species[b.species].size as usize;
            let (bx, by) = b.tile();
            if (bx..bx + n).contains(&cx) && (by..by + n).contains(&cy) {
                v += 6;
            }
        }
    }
    v
}


/// Plans that push a crate onto a switch nobody holds: a search over pushes, the
/// walk to each push made round every crate.
fn crate_plans(g: &Game, me: At) -> Vec<Plan> {
    let switches: Vec<At> = (0..g.h)
        .flat_map(|y| (0..g.w).map(move |x| (x, y)))
        .filter(|&(x, y)| g.ground[y][x] == Ground::Switch && g.tiles[y][x] == Tile::Floor && !g.occupied_pub(x, y))
        .collect();
    if switches.is_empty() {
        return Vec::new();
    }
    let crates: Vec<At> = (0..g.h)
        .flat_map(|y| (0..g.w).map(move |x| (x, y)))
        .filter(|&(x, y)| g.tiles[y][x] == Tile::Block && g.ground[y][x] != Ground::Switch)
        .collect();
    let floor = |x: i32, y: i32, crate_at: At| -> bool {
        x > 0
            && y > 0
            && (x as usize) < g.w
            && (y as usize) < g.h
            && (x as usize, y as usize) != crate_at
            && g.tiles[y as usize][x as usize] == Tile::Floor
            && g.bomb_at(x as usize, y as usize).is_none()
    };
    // walking paths with crates as walls and `c` where the moving crate stands
    let walk = |from: At, c: At| -> Vec<Vec<Option<At>>> {
        let mut par = vec![vec![None; g.w]; g.h];
        par[from.1][from.0] = Some(from);
        let mut q = VecDeque::from([from]);
        while let Some((x, y)) = q.pop_front() {
            for (dx, dy) in [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if floor(nx, ny, c) && par[ny as usize][nx as usize].is_none() {
                    par[ny as usize][nx as usize] = Some((x, y));
                    q.push_back((nx as usize, ny as usize));
                }
            }
        }
        par
    };
    let mut plans = Vec::new();
    for &c0 in &crates {
        // state: crate tile and player tile (beside it); parent links for the plan
        let mut seen: std::collections::HashMap<(At, At), Option<(At, At)>> = std::collections::HashMap::new();
        seen.insert((c0, me), None);
        let mut q = VecDeque::from([(c0, me)]);
        let mut goal = None;
        while let Some((c, pl)) = q.pop_front() {
            if switches.contains(&c) {
                goal = Some((c, pl));
                break;
            }
            if seen.len() > 4000 {
                break;
            }
            let reach = walk(pl, c);
            for (dx, dy) in [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
                let behind = (c.0 as i32 - dx, c.1 as i32 - dy);
                let ahead = (c.0 as i32 + dx, c.1 as i32 + dy);
                if !floor(behind.0, behind.1, c) || !floor(ahead.0, ahead.1, c) {
                    continue;
                }
                let behind = (behind.0 as usize, behind.1 as usize);
                if reach[behind.1][behind.0].is_none() || g.ground[ahead.1 as usize][ahead.0 as usize] == Ground::Warp {
                    continue;
                }
                let next = ((ahead.0 as usize, ahead.1 as usize), c);
                if !seen.contains_key(&next) {
                    seen.insert(next, Some((c, pl)));
                    q.push_back(next);
                }
            }
        }
        let Some(mut at) = goal else { continue };
        let mut chain = vec![at];
        while let Some(Some(prev)) = seen.get(&at) {
            chain.push(*prev);
            at = *prev;
        }
        chain.reverse();
        // each push: walk behind the crate, then lean into it
        let mut acts = Vec::new();
        for w in chain.windows(2) {
            let ((c, pl), (c2, _)) = (w[0], w[1]);
            let behind = (2 * c.0 - c2.0, 2 * c.1 - c2.1);
            let par = walk(pl, c);
            if let Some(path) = path_to(&par, pl, behind) {
                if path.len() > 1 {
                    acts.push(Act::Go(path));
                }
                acts.push(Act::Go(vec![behind, c]));
            }
        }
        if !acts.is_empty() {
            plans.push(Plan::new(acts, "crate"));
        }
    }
    plans
}

pub struct Bot {
    plan: Option<Plan>,
    pub decisions: u32,
    /// What the plan being played is, and whether it was a safe one.
    pub doing: (&'static str, bool),
    /// Varies close choices, so a retried stage is not played the same way again.
    pub noise: u64,
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot {
    pub fn new() -> Bot {
        Bot { plan: None, decisions: 0, doing: ("", true), noise: 0 }
    }

    /// Player `i`'s command for this step of a stage being played.
    pub fn cmd(&mut self, g: &Game, i: usize) -> Cmd {
        let p = &g.players[i];
        if !p.alive {
            self.plan = None;
            return Cmd::default();
        }
        for _ in 0..2 {
            if self.plan.is_none() {
                self.plan = Some(self.choose(g, i));
                self.decisions += 1;
            }
            if let Some(c) = self.plan.as_mut().and_then(|pl| pl.cmd(g, i)) {
                return c;
            }
            self.plan = None;
        }
        Cmd::default()
    }

    fn choose(&mut self, g: &Game, i: usize) -> Plan {
        let p = &g.players[i];
        let me = p.tile();
        let speed = 12 + 3 * p.speed;
        let par = paths(g, i, me, None);
        let danger = danger(g);
        let mut cands: Vec<Plan> = Vec::new();
        // the open hatch
        if g.exit_open
            && let Some(e) = g.exit
            && let Some(path) = path_to(&par, me, e)
        {
            cands.push(Plan::new(vec![Act::Go(path)], "exit"));
        }
        // things to pick up: keys, parts, items, deleted files to empty, eggs
        let mut picks: Vec<(usize, At)> = Vec::new();
        for y in 0..g.h {
            for x in 0..g.w {
                let good = g.pickups[y][x].is_some()
                    || g.items[y][x].is_some_and(|it| worth(it) > 0 && g.tiles[y][x] == Tile::Floor);
                if good && par[y][x].is_some() {
                    picks.push((dist(me, (x, y)), (x, y)));
                }
            }
        }
        for r in &g.restores {
            if par[r.y][r.x].is_some() {
                picks.push((dist(me, (r.x, r.y)), (r.x, r.y)));
            }
        }
        picks.sort();
        for &(_, t) in picks.iter().take(3) {
            if let Some(path) = path_to(&par, me, t) {
                cands.push(Plan::new(vec![Act::Go(path)], "pick up"));
            }
        }
        // bomb spots: the best few near, and the best anywhere
        let free = g.bombs.iter().filter(|b| b.owner == i).count() < p.bombs as usize;
        if free {
            let mut spots: Vec<(i32, usize, At)> = Vec::new();
            for y in 0..g.h {
                for x in 0..g.w {
                    if par[y][x].is_none() || g.tiles[y][x] != Tile::Floor || g.bomb_at(x, y).is_some() {
                        continue;
                    }
                    let v = bomb_value(g, i, x, y);
                    if v > 0 {
                        spots.push((v, dist(me, (x, y)), (x, y)));
                    }
                }
            }
            let mut near = spots.clone();
            near.sort_by_key(|s| (s.1 as i32 * 2 - s.0, s.1));
            spots.sort_by_key(|s| (-s.0, s.1));
            let mut chosen: Vec<At> = near.iter().take(6).map(|s| s.2).collect();
            for s in spots.iter().take(3) {
                if !chosen.contains(&s.2) {
                    chosen.push(s.2);
                }
            }
            // lie in wait: in line with a creature near by, or right here
            let mut lines: Vec<(usize, At)> = Vec::new();
            for b in g.baddies.iter().filter(|b| b.alive) {
                let bt = b.tile();
                if dist(me, bt) > 9 {
                    continue;
                }
                for y in 0..g.h {
                    for x in 0..g.w {
                        let inline = (x == bt.0 && y.abs_diff(bt.1) <= p.fire as usize) || (y == bt.1 && x.abs_diff(bt.0) <= p.fire as usize);
                        if inline && (x, y) != bt && par[y][x].is_some() && g.tiles[y][x] == Tile::Floor && g.bomb_at(x, y).is_none() {
                            lines.push((dist(me, (x, y)), (x, y)));
                        }
                    }
                }
            }
            lines.sort();
            for &(_, t) in lines.iter().take(4) {
                if !chosen.contains(&t) {
                    chosen.push(t);
                }
            }
            if g.tiles[me.1][me.0] == Tile::Floor && g.bomb_at(me.0, me.1).is_none() && !chosen.contains(&me) {
                chosen.push(me);
            }
            for t in chosen {
                let Some(go) = path_to(&par, me, t) else { continue };
                // ways out of the blast from there
                let pierce = p.kind == crate::game::BombKind::Pierce;
                let mut blast = cross(g, t.0, t.1, p.fire, pierce);
                // a charge it reaches goes off with a blast of its own
                for k in 0..blast.len() {
                    let (cx, cy) = blast[k];
                    if g.tiles[cy][cx] == Tile::Dud {
                        blast.extend(cross(g, cx, cy, 3, false));
                    }
                }
                let out = paths(g, i, t, Some(t));
                let mut exits: Vec<(usize, At)> = Vec::new();
                for y in 0..g.h {
                    for x in 0..g.w {
                        if out[y][x].is_some() && !blast.contains(&(x, y)) && !danger[y][x] {
                            exits.push((dist(t, (x, y)), (x, y)));
                        }
                    }
                }
                exits.sort();
                for &(_, e) in exits.iter().take(4) {
                    let mut back = path_to(&out, t, e).unwrap_or_default();
                    if back.is_empty() {
                        continue;
                    }
                    // the escape starts where the walk ends
                    if back[0] != t {
                        back.insert(0, t);
                    }
                    let mut acts = vec![Act::Go(go.clone()), Act::Bomb, Act::Go(back)];
                    if p.kind == crate::game::BombKind::Remote {
                        acts.push(Act::Wait(20));
                        acts.push(Act::Fire);
                    }
                    cands.push(Plan::new(acts, "bomb"));
                }
            }
        }
        // a crate onto a switch
        cands.extend(crate_plans(g, me));
        // toward the nearest creature, to meet it
        let mut prey: Vec<(usize, At)> = g
            .baddies
            .iter()
            .filter(|b| b.alive)
            .map(|b| (dist(me, b.tile()), b.tile()))
            .collect();
        prey.sort();
        let hunting = matches!(g.goal, Goal::Clear | Goal::Boss);
        if hunting && let Some(&(_, bt)) = prey.first() {
            let mut best: Option<(usize, At)> = None;
            for y in 0..g.h {
                for x in 0..g.w {
                    let d = dist((x, y), bt);
                    if par[y][x].is_some() && g.tiles[y][x] == Tile::Floor && d >= 3 && best.is_none_or(|b| d < b.0) {
                        best = Some((d, (x, y)));
                    }
                }
            }
            if let Some((_, t)) = best
                && t != me
                && let Some(path) = path_to(&par, me, t)
            {
                cands.push(Plan::new(vec![Act::Go(path)], "approach"));
            }
        }
        // out of every blast
        let mut safe: Vec<(usize, At)> = Vec::new();
        for y in 0..g.h {
            for x in 0..g.w {
                if par[y][x].is_some() && g.tiles[y][x] == Tile::Floor && !danger[y][x] {
                    safe.push((dist(me, (x, y)), (x, y)));
                }
            }
        }
        safe.sort();
        for &(_, t) in safe.iter().take(3) {
            if t != me
                && let Some(path) = path_to(&par, me, t)
            {
                cands.push(Plan::new(vec![Act::Go(path)], "shelter"));
            }
        }
        // somewhere far, to look round
        if let Some(&(_, t)) = safe.iter().rev().find(|s| s.0 > 3)
            && let Some(path) = path_to(&par, me, t)
        {
            cands.push(Plan::new(vec![Act::Go(path)], "wander"));
        }
        cands.push(Plan::new(vec![Act::Wait(10)], "wait"));

        let ticking = g.bombs.iter().map(|b| b.fuse).max().unwrap_or(0);
        let mut best: Option<(i64, Plan)> = None;
        let mut fallback: Option<(u32, Plan)> = None;
        let ncand = cands.len();
        // what happens anyway, standing still: a plan only earns what it adds
        let mut base = Vec::new();
        simulate(g, &Plan::new(vec![Act::Wait(1)], "base"), i, 900, Some(&mut base));
        let base_at = |h: u32| base.get(h as usize).or(base.last()).copied().unwrap_or(0);
        for (k, c) in cands.into_iter().enumerate() {
            let len = c.length(speed);
            let tail = if c.bombs() { FUSE + FLAME + 20 } else { (ticking + FLAME + 10).max(70) };
            let horizon = (len.max(c.limit) + tail).min(900);
            let mut o = simulate(g, &c, i, horizon, None);
            o.gain -= base_at(horizon.saturating_sub(1));
            if !o.safe {
                if fallback.as_ref().is_none_or(|f| o.steps > f.0) {
                    fallback = Some((o.steps, c));
                }
                continue;
            }
            let score = if o.cleared {
                1_000_000 - o.steps as i64
            } else {
                let base = match c.what {
                    "approach" => 1,
                    "crate" => 4,
                    "wander" => 0,
                    _ => 0,
                };
                (o.gain + base) as i64 * 10_000 / (o.steps as i64 + 40)
            };
            let score = if self.noise > 0 {
                let h = (self.noise ^ (k as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ g.clock as u64).wrapping_mul(0xff51_afd7_ed55_8ccd);
                score + (h >> 58) as i64 * 4
            } else {
                score
            };
            if best.as_ref().is_none_or(|b| score > b.0) {
                best = Some((score, c));
            }
        }
        if std::env::var_os("UXO_BOT_TRACE").is_some() {
            eprintln!(
                "t{} at {:?} bombs {}/{} fire {}: {} safe {:?}, fallback {:?}",
                g.clock / 60,
                me,
                g.bombs.iter().filter(|b| b.owner == i).count(),
                p.bombs,
                p.fire,
                ncand,
                best.as_ref().map(|b| (b.0, b.1.what, b.1.acts.iter().map(|a| match a { Act::Go(v) => format!("go{:?}", v.last()), a => format!("{a:?}") }).collect::<Vec<_>>())),
                fallback.as_ref().map(|f| (f.0, f.1.what))
            );
        }
        self.doing = (best.as_ref().or(fallback.as_ref().map(|_| &best).and(None)).map_or("", |b| b.1.what), best.is_some());
        match (best, fallback) {
            (Some((_, c)), _) => c,
            (None, Some((_, c))) => {
                self.doing = (c.what, false);
                c
            }
            (None, None) => Plan::new(vec![Act::Wait(10)], "wait"),
        }
    }
}

/// How one stage went for the bot.
#[derive(Debug, Default, Clone)]
pub struct Report {
    pub code: String,
    pub name: String,
    pub cleared: bool,
    pub lives_lost: u32,
    pub secs: u32,
    pub time: u32,
    pub notes: Vec<String>,
}

/// The bot plays adventure stage `index` from a fresh start (three lives, no
/// power-ups carried), up to `tries` lives, and says how it went.
pub fn play_stage(index: usize, seed: u64) -> Report {
    let mut r = Run::new(seed, 1, index);
    // `UXO_BOT_KIT=bombs,fire,speed`: start with power-ups, as a player arriving would
    if let Ok(kit) = std::env::var("UXO_BOT_KIT") {
        let v: Vec<u32> = kit.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        let p = &mut r.game.players[0];
        if let [b, f, sp] = v[..] {
            (p.bombs, p.fire, p.speed) = (b, f, sp as i32);
        }
    }
    let mut bot = Bot::new();
    let mut rep = Report {
        code: adventure::code_of(index),
        name: r.stage.name.clone(),
        time: r.stage.time,
        ..Report::default()
    };
    let start_lives = r.lives[0];
    let mut played = 0u32;
    for _ in 0..(r.stage.time + 30) * 60 * 4 {
        let mut cmds = [Cmd::default(); MAX_PLAYERS];
        match r.phase {
            Phase::Card(_) => {
                cmds[0].bomb = true;
                // a stage begun again is a new board: plans for the old one are void
                bot.plan = None;
            }
            Phase::Play => {
                cmds[0] = bot.cmd(&r.game, 0);
                played += 1;
            }
            Phase::Clear { .. } => {
                rep.cleared = true;
                break;
            }
            Phase::GameOver | Phase::Done | Phase::Ending(_) | Phase::Map { .. } => break,
            Phase::TimeUp { .. } => {
                if !rep.notes.last().is_some_and(|n| n.starts_with("time up") || n.starts_with("Payload")) {
                    let g = &r.game;
                    let duds: Vec<At> = (0..g.h).flat_map(|y| (0..g.w).map(move |x| (x, y))).filter(|&(x, y)| g.tiles[y][x] == Tile::Dud).collect();
                    let left = g.baddies.iter().filter(|b| b.alive).count();
                    rep.notes.push(format!(
                        "time up after {}s (duds {:?}, creatures {left}, hatch {}, at {:?})",
                        played / 60,
                        duds,
                        if g.exit_open { "open" } else { "shut" },
                        g.players[0].tile()
                    ));
                }
            }
        }
        let events = r.step(&cmds);
        for ev in events {
            if let Event::Death { player: 0, .. } = ev {
                let p = &r.game.players[0];
                rep.notes.push(format!("caught at {:?} after {}s ({} {})", p.tile(), played / 60, bot.doing.0, if bot.doing.1 { "safe" } else { "cornered" }));
            }
            if ev == Event::Cleared {
                rep.cleared = true;
            }
            if ev == Event::Enter {
                rep.notes.push(format!("Payload pressed Enter after {}s", played / 60));
            }
        }
        if matches!(r.phase, Phase::Clear { .. }) {
            rep.cleared = true;
            break;
        }
        if r.index != index {
            break;
        }
    }
    rep.lives_lost = start_lives - r.lives[0].min(start_lives);
    rep.secs = played / 60;
    if !rep.cleared {
        rep.notes.push(format!("not cleared: {:?}, {} decisions", r.phase, bot.decisions));
    }
    rep
}

/// The whole adventure from 1-1 by the bot, carrying power-ups as a player would, the
/// map choosing its first offer. Logs every stage; says how far it got.
pub fn play_through(seed: u64, from: usize, log: &mut dyn FnMut(String)) -> bool {
    let mut r = Run::new(seed, 1, from);
    let mut bot = Bot::new();
    let mut stage = usize::MAX;
    let mut played = 0u32;
    let mut game_overs = 0;
    let mut lives_at = r.lives[0];
    loop {
        if r.index != stage {
            stage = r.index;
            played = 0;
            lives_at = r.lives[0];
            log(format!("{} {}", r.code(), r.stage.name));
        }
        let mut cmds = [Cmd::default(); MAX_PLAYERS];
        match r.phase {
            Phase::Card(_) | Phase::Ending(_) | Phase::Map { .. } => {
                cmds[0].bomb = played % 30 == 29;
                bot.plan = None;
            }
            Phase::TimeUp { .. } => bot.plan = None,
            Phase::Play => cmds[0] = bot.cmd(&r.game, 0),
            Phase::GameOver => {
                game_overs += 1;
                bot.noise = game_overs;
                log(format!("  game over on {} ({} so far)", r.code(), game_overs));
                if game_overs > 40 {
                    return false;
                }
                cmds[0].bomb = true;
            }
            Phase::Done => {
                log(format!("THE END, {game_overs} game overs"));
                return true;
            }
            _ => {}
        }
        played += 1;
        let before = r.phase;
        for ev in r.step(&cmds) {
            if let Event::Death { player: 0, .. } = ev {
                log(format!("  caught at {:?} after {}s", r.game.players[0].tile(), played / 60));
            }
        }
        if matches!(r.phase, Phase::Clear { .. }) && !matches!(before, Phase::Clear { .. }) {
            log(format!(
                "  cleared in {}s of {}s, lives {} (were {}), bombs {} fire {} speed {}",
                (r.stage.time * 60 - r.left) / 60,
                r.stage.time,
                r.lives[0],
                lives_at,
                r.game.players[0].bombs,
                r.game.players[0].fire,
                r.game.players[0].speed
            ));
        }
        if r.game.goal == Goal::None {
            return false;
        }
    }
}
