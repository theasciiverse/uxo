//! Battle stages: original maps in `stages/*.txt`, compiled in.
//!
//! Format: header lines (`name:`, `about:`, optional `give: kick, glove`, `items: 35`
//! percent of soft blocks hiding an item), then `grid:` and the map, any size (at least
//! 7x7, every row the same length, walled all round), one character per tile:
//!
//! - `#` wall, `+` soft block, `?` soft block three times in four, `.` floor
//! - `1` to `4` player starts
//! - `>` `<` `^` `v` conveyor belts (carry players and bombs)
//! - `R` `L` `U` `D` arrows (turn sliding bombs)
//! - `o` warp (each sends you to the next one in reading order)
//! - `~` ice (you slide until something stops you)
//! - `E` the exit hatch, `*` a dud (adventure)
//! - `w` water and `T` trees: solid like walls, drawn as a pond or a canopy
//! - a letter from a `baddie: g grub` legend line: that baddie starts there
//!
//! Adventure stages also have `world:`, `story:` lines (the card before the stage),
//! `goal:` clear | duds | boss, `time:` seconds, `theme:` brick | orchard, and
//! `ending:` lines (after a world's last stage).

use crate::game::{Dir, Goal, Item, Theme};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Cell {
    Wall,
    Soft,
    MaybeSoft,
    Floor,
    Start(usize),
    Belt(Dir),
    Arrow(Dir),
    Warp,
    Ice,
    Exit,
    Dud,
    Water,
    Tree,
    Baddie(String),
}

#[derive(Clone, Debug)]
pub struct Stage {
    pub name: String,
    pub about: String,
    pub give: Vec<Item>,
    pub items: u32,
    pub world: String,
    pub story: Vec<String>,
    pub ending: Vec<String>,
    pub goal: Goal,
    /// Seconds on the clock (adventure).
    pub time: u32,
    pub theme: Theme,
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Vec<Cell>>,
}

pub const FILES: [&str; 6] = [
    include_str!("../stages/01-open-field.txt"),
    include_str!("../stages/02-conveyor-works.txt"),
    include_str!("../stages/03-ricochet.txt"),
    include_str!("../stages/04-warp-garden.txt"),
    include_str!("../stages/05-ice-rink.txt"),
    include_str!("../stages/06-catacombs.txt"),
];

pub fn count() -> usize {
    FILES.len()
}

pub fn load(i: usize) -> Stage {
    parse(FILES[i % FILES.len()]).unwrap_or_else(|e| panic!("stage {}: {e}", i + 1))
}

fn item_named(s: &str) -> Option<Item> {
    Some(match s.trim() {
        "kick" => Item::Kick,
        "punch" => Item::Punch,
        "glove" => Item::Glove,
        "line" => Item::LineBomb,
        "remote" => Item::Remote,
        "pierce" => Item::Pierce,
        "fire" => Item::FireUp,
        "bomb" => Item::BombUp,
        "speed" => Item::SpeedUp,
        "heart" => Item::Heart,
        _ => return None,
    })
}

pub fn parse(text: &str) -> Result<Stage, String> {
    let mut st = Stage {
        name: String::new(),
        about: String::new(),
        give: Vec::new(),
        items: 35,
        world: String::new(),
        story: Vec::new(),
        ending: Vec::new(),
        goal: Goal::None,
        time: 180,
        theme: Theme::Brick,
        w: 0,
        h: 0,
        cells: Vec::new(),
    };
    let mut legend: Vec<(char, String)> = Vec::new();
    let mut lines = text.lines();
    for line in lines.by_ref() {
        let Some((k, v)) = line.split_once(':') else { continue };
        let v = v.trim();
        match k.trim() {
            "name" => st.name = v.to_string(),
            "about" => st.about = v.to_string(),
            "give" => {
                for w in v.split(',') {
                    st.give.push(item_named(w).ok_or(format!("unknown item {w:?}"))?);
                }
            }
            "items" => st.items = v.parse().map_err(|_| format!("bad items {v:?}"))?,
            "world" => st.world = v.to_string(),
            "story" => st.story.push(v.to_string()),
            "ending" => st.ending.push(v.to_string()),
            "time" => st.time = v.parse().map_err(|_| format!("bad time {v:?}"))?,
            "goal" => {
                st.goal = match v {
                    "clear" => Goal::Clear,
                    "duds" => Goal::Duds,
                    "boss" => Goal::Boss,
                    _ => return Err(format!("bad goal {v:?}")),
                }
            }
            "theme" => {
                st.theme = match v {
                    "brick" => Theme::Brick,
                    "orchard" => Theme::Orchard,
                    _ => return Err(format!("bad theme {v:?}")),
                }
            }
            "baddie" => {
                let (c, id) = v.split_once(' ').ok_or(format!("bad baddie line {v:?}"))?;
                let c = c.chars().next().filter(|c| c.is_ascii_lowercase() && !"ovw".contains(*c)).ok_or(format!("baddie letter {c:?} must be lower case, not o, v or w"))?;
                legend.push((c, id.trim().to_string()));
            }
            "grid" => break,
            other => return Err(format!("unknown header {other:?}")),
        }
    }
    let rows: Vec<&str> = lines.filter(|l| !l.trim().is_empty()).collect();
    st.h = rows.len();
    st.w = rows.first().map_or(0, |r| r.chars().count());
    if st.w < 7 || st.h < 7 {
        return Err(format!("grid is {}x{}, want at least 7x7", st.w, st.h));
    }
    for (y, row) in rows.iter().enumerate() {
        let chars: Vec<char> = row.chars().collect();
        if chars.len() != st.w {
            return Err(format!("row {} has {} cells, want {}", y + 1, chars.len(), st.w));
        }
        let mut out = Vec::with_capacity(st.w);
        for (x, &c) in chars.iter().enumerate() {
            let edge = x == 0 || y == 0 || x == st.w - 1 || y == st.h - 1;
            if edge && c != '#' {
                return Err(format!("the border must be wall, found {c:?} at {x},{y}"));
            }
            out.push(match c {
                '#' => Cell::Wall,
                '+' => Cell::Soft,
                '?' => Cell::MaybeSoft,
                '.' => Cell::Floor,
                '1'..='4' => Cell::Start(c as usize - '1' as usize),
                '>' => Cell::Belt(Dir::Right),
                '<' => Cell::Belt(Dir::Left),
                '^' => Cell::Belt(Dir::Up),
                'v' => Cell::Belt(Dir::Down),
                'R' => Cell::Arrow(Dir::Right),
                'L' => Cell::Arrow(Dir::Left),
                'U' => Cell::Arrow(Dir::Up),
                'D' => Cell::Arrow(Dir::Down),
                'o' => Cell::Warp,
                '~' => Cell::Ice,
                'E' => Cell::Exit,
                'w' => Cell::Water,
                'T' => Cell::Tree,
                '*' => Cell::Dud,
                _ => match legend.iter().find(|(k, _)| *k == c) {
                    Some((_, id)) => Cell::Baddie(id.clone()),
                    None => return Err(format!("unknown cell {c:?} at {x},{y}")),
                },
            });
        }
        st.cells.push(out);
    }
    // battle maps seat four; adventure stages two (co-op)
    let seats = if st.goal == Goal::None { 4 } else { 2 };
    for i in 0..seats {
        if !st.cells.iter().flatten().any(|c| *c == Cell::Start(i)) {
            return Err(format!("no start for player {}", i + 1));
        }
    }
    if st.goal != Goal::None && st.goal != Goal::Boss && !st.cells.iter().flatten().any(|c| *c == Cell::Exit) {
        return Err("an adventure stage needs an exit (E)".into());
    }
    Ok(st)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stage_parses() {
        for i in 0..count() {
            let s = load(i);
            assert!(!s.name.is_empty(), "stage {} has no name", i + 1);
        }
    }

    #[test]
    fn every_stage_is_connected() {
        for i in 0..count() {
            assert_connected(&load(i));
        }
    }
}

/// Every open or breakable tile can be reached from player 1's start (walking through
/// soft blocks, which can be blown up), so no map has a sealed pocket, and every start
/// has a free tile beside it to dodge into.
#[cfg(test)]
pub fn assert_connected(s: &Stage) {
    let open = |x: usize, y: usize| !matches!(s.cells[y][x], Cell::Wall | Cell::Water | Cell::Tree);
    let (w, h) = (s.w, s.h);
    let start = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| s.cells[y][x] == Cell::Start(0)).unwrap();
    let mut seen = vec![vec![false; w]; h];
    let mut todo = vec![start];
    while let Some((x, y)) = todo.pop() {
        if seen[y][x] {
            continue;
        }
        seen[y][x] = true;
        for (dx, dy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h && open(nx as usize, ny as usize) {
                todo.push((nx as usize, ny as usize));
            }
        }
    }
    for y in 0..h {
        for x in 0..w {
            assert!(!open(x, y) || seen[y][x], "{}: tile {x},{y} can't be reached", s.name);
            if let Cell::Start(p) = s.cells[y][x] {
                let free = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)]
                    .iter()
                    .any(|&(dx, dy)| s.cells[(y as i32 + dy) as usize][(x as i32 + dx) as usize] == Cell::Floor);
                assert!(free, "{}: start {} is boxed in", s.name, p + 1);
            }
        }
    }
}
