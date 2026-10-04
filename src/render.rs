//! Drawing the arena. Pixel sizes are drawn in half-blocks (`▀`, fg = top pixel,
//! bg = bottom, so a terminal cell is two roughly square pixels). A tile is `px`
//! pixels square: hand-drawn art at 4, 6 and 8, and whole-number upscales of the 6
//! and 8 art above that (12, 16, 18, 24), so art pixels never come out uneven. The
//! smallest size is one glyph pair per tile (2x1 cells) for tiny terminals. Players
//! move in pixels, not tiles, so movement stays smooth at every pixel size.

use crate::baddie::{Species, State};
use crate::game::{Bomb, BombKind, Dir, Disease, Flame, Game, Ground, Item, Motion, Player, Theme, Tile, DOWN, DROP, FUSE, LEFT, RIGHT, TILE, UP};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

/// Tile size in pixels; 0 is the glyph fallback.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Zoom(i32);

const SIZES: [i32; 8] = [0, 4, 6, 8, 12, 16, 18, 24];

impl Zoom {
    pub const SMALL: Zoom = Zoom(0);

    pub fn px(self) -> i32 {
        self.0
    }
    /// Terminal cells per tile.
    pub fn cells(self) -> (u16, u16) {
        if self.0 == 0 { (2, 1) } else { (self.0 as u16, self.0 as u16 / 2) }
    }
    /// How many tiles a view of `w` x `h` cells shows at this size.
    pub fn tiles_in(self, w: u16, h: u16) -> (f32, f32) {
        let (cw, ch) = self.cells();
        (w as f32 / cw as f32, h as f32 / ch as f32)
    }
    /// The pixel sizes, largest first.
    pub fn pixel_sizes() -> impl Iterator<Item = Zoom> {
        SIZES.iter().rev().filter(|&&s| s > 0).map(|&s| Zoom(s))
    }
    pub fn step(self, by: i32) -> Zoom {
        let i = SIZES.iter().position(|&s| s == self.0).unwrap_or(0) as i32;
        Zoom(SIZES[(i + by).clamp(0, SIZES.len() as i32 - 1) as usize])
    }
    /// Parse a size for the command line: a pixel count, or `s` for glyphs.
    pub fn parse(s: &str) -> Option<Zoom> {
        if s == "s" {
            return Some(Zoom::SMALL);
        }
        s.parse().ok().filter(|v| SIZES.contains(v)).map(Zoom)
    }
    pub fn name(self) -> String {
        if self.0 == 0 { "glyphs".into() } else { format!("{}px", self.0) }
    }
    /// Which hand-drawn art set, and how many times it is scaled up.
    fn art(self) -> (i32, i32) {
        match self.0 {
            s if s % 8 == 0 => (8, s / 8),
            s if s % 6 == 0 => (6, s / 6),
            s => (4, s / 4),
        }
    }
}

pub type Rgb = [u8; 3];

/// Player colours. Amber first: the Asciiverse accent.
pub const PLAYER: [Rgb; 4] = [[255, 176, 0], [60, 200, 235], [225, 85, 205], [150, 220, 60]];

const FLOOR_A: Rgb = [58, 138, 62];
const FLOOR_B: Rgb = [50, 124, 55];
const FLOOR_SHADOW: Rgb = [36, 92, 42];

fn shade(c: Rgb, f: f32) -> Rgb {
    c.map(|v| (v as f32 * f).min(255.0) as u8)
}

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    std::array::from_fn(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t) as u8)
}

/// Cheap per-pixel noise for flicker and crumbling.
fn hash(a: i32, b: i32, c: u64) -> u32 {
    let mut h = (a as u32).wrapping_mul(0x9e37_79b1) ^ (b as u32).wrapping_mul(0x85eb_ca77) ^ (c as u32).wrapping_mul(0xc2b2_ae3d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^ (h >> 12)
}

/// A pixel buffer looking at the world: drawing takes world pixel coordinates, and
/// whatever falls outside the window at (`ox`, `oy`) is dropped.
struct Canvas {
    w: i32,
    h: i32,
    ox: i32,
    oy: i32,
    px: Vec<Rgb>,
}

impl Canvas {
    fn new(w: i32, h: i32) -> Canvas {
        Canvas::window(w, h, 0, 0, [0, 0, 0])
    }
    fn window(w: i32, h: i32, ox: i32, oy: i32, bg: Rgb) -> Canvas {
        Canvas { w, h, ox, oy, px: vec![bg; (w.max(0) * h.max(0)) as usize] }
    }
    fn set(&mut self, x: i32, y: i32, c: Rgb) {
        let (x, y) = (x - self.ox, y - self.oy);
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }
    /// Stamp a sprite: one char per art pixel, each drawn `k` x `k`; '.' is transparent.
    fn sprite(&mut self, x: i32, y: i32, k: i32, rows: &[&str], pal: impl Fn(char) -> Option<Rgb>) {
        for (dy, row) in rows.iter().enumerate() {
            for (dx, ch) in row.chars().enumerate() {
                if let Some(c) = pal(ch) {
                    self.block(x + dx as i32 * k, y + dy as i32 * k, k, c);
                }
            }
        }
    }
    fn block(&mut self, x: i32, y: i32, k: i32, c: Rgb) {
        for yy in 0..k {
            for xx in 0..k {
                self.set(x + xx, y + yy, c);
            }
        }
    }
}

// Hand-drawn art at 4, 6 and 8 pixels per tile. Placeholder art: the hero and the
// items are ours to design properly later (see CLAUDE.md "Power-ups").
fn wall(a: i32) -> &'static [&'static str] {
    match a {
        8 => &["hhhhhhhd", "hhgggggd", "hggggggd", "hggggggd", "hggggggd", "hggggggd", "hgggggdd", "dddddddd"],
        6 => &["hhhhhd", "hggggd", "hggggd", "hggggd", "hggggd", "dddddd"],
        _ => &["hhhd", "hggd", "hggd", "dddd"],
    }
}

fn soft(a: i32) -> &'static [&'static str] {
    match a {
        8 => &["lllrllll", "bbbrbbbb", "bbbrbbbb", "rrrrrrrr", "llllllrl", "bbbbbbrb", "bbbbbbrb", "rrrrrrrr"],
        6 => &["llrlll", "bbrbbb", "rrrrrr", "bbbbrb", "bbbbrb", "rrrrrr"],
        _ => &["lrll", "rrrr", "bbrb", "rrrr"],
    }
}

/// A pond tile: deep blue with ripples drifting across.
fn water(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, t: u64) {
    for v in 0..a {
        for u in 0..a {
            let (x, y) = (x0 + u * k, y0 + v * k);
            let wave = ((x / k + y / k * 2) as f32 * 0.6 + t as f32 / 12.0).sin();
            let c = if wave > 0.85 {
                [150, 200, 240]
            } else if wave > 0.2 {
                [60, 120, 200]
            } else {
                [45, 95, 175]
            };
            cv.block(x, y, k, c);
        }
    }
}

/// A tree's canopy: clumps of dark and light leaves, the same from frame to frame.
fn canopy(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, tx: usize, ty: usize) {
    for v in 0..a {
        for u in 0..a {
            let (x, y) = (x0 + u * k, y0 + v * k);
            let h = hash(x / k + tx as i32 * 3, y / k + ty as i32 * 5, 17) % 10;
            let c = match h {
                0..=2 => [30, 80, 35],
                3..=6 => [45, 110, 45],
                7 | 8 => [70, 140, 60],
                _ => [110, 175, 80],
            };
            cv.block(x, y, k, c);
        }
    }
}

/// An Orchard hedge: leaves and a few apples.
fn hedge(a: i32) -> &'static [&'static str] {
    match a {
        8 => &["dgglggld", "gllgrlgg", "glgggllg", "lggrgggl", "gglggrgg", "glrgglgg", "ggglgggd", "dddddddd"],
        6 => &["dglggd", "gllrlg", "lgggrl", "grglgg", "gglggd", "dddddd"],
        _ => &["glgd", "lrgl", "gglr", "dddd"],
    }
}

/// A dud: an unexploded firework rocket standing on its fins.
fn dud(a: i32) -> &'static [&'static str] {
    match a {
        8 => &["...yy...", "..yrry..", "..rrrr..", "..wwww..", "..rrrr..", "..rrrr..", ".brrrrb.", ".b.ff.b."],
        6 => &["..yy..", ".yrry.", ".wwww.", ".rrrr.", "brrrrb", "b.ff.b"],
        _ => &[".yy.", ".rr.", ".ww.", "brrb"],
    }
}

/// The exit hatch: a dark ring, glowing amber once the goal is met.
fn exit_hatch(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, open: bool, t: u64) {
    let c = (a as f32 - 1.0) / 2.0;
    let pulse = 0.5 + 0.5 * ((t as f32) / 6.0).sin();
    for v in 0..a {
        for u in 0..a {
            let r = (u as f32 - c).hypot(v as f32 - c);
            let col = if r > c + 0.3 {
                continue;
            } else if r > c - 0.9 {
                if open { [255, 200, 80] } else { [70, 70, 80] }
            } else if open {
                mix([255, 150, 40], [255, 250, 210], pulse * (1.0 - r / c))
            } else {
                [25, 25, 32]
            };
            cv.block(x0 + u * k, y0 + v * k, k, col);
        }
    }
}

/// An egg with spots in its species' colour.
fn draw_egg(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, spot: Rgb) {
    let rows: &[&str] = match a {
        8 => &["...ww...", "..wwww..", ".wwswww.", ".wwwwsw.", ".wswwww.", ".wwwwww.", "..wwww..", "........"],
        6 => &["..ww..", ".wwsw.", ".wsww.", ".wwww.", "..ww..", "......"],
        _ => &[".ww.", "wsww", "wwsw", ".ww."],
    };
    cv.sprite(x0, y0, k, rows, |c| match c {
        'w' => Some([245, 240, 225]),
        's' => Some(spot),
        _ => None,
    });
}

fn egg_spot(sp: &Species) -> Rgb {
    sp.colors.first().map_or([200, 100, 100], |c| c.1)
}

/// Draw a species' art, scaled from its 8 (or 16) pixel rows to `a` art pixels per
/// tile, `turns` quarter turns rolled, darker when curled, mirrored when facing left.
#[allow(clippy::too_many_arguments)]
fn draw_species(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, sp: &Species, art: &[String], turns: usize, dark: bool, flip: bool) {
    let src = 8 * sp.size as usize;
    let dst = (a * sp.size) as usize;
    let rows: Vec<Vec<char>> = art.iter().map(|r| r.chars().collect()).collect();
    for v in 0..dst {
        for u in 0..dst {
            // nearest source pixel, then turn and mirror
            let (mut su, mut sv) = (u * src / dst, v * src / dst);
            for _ in 0..turns {
                (su, sv) = (sv, src - 1 - su);
            }
            if flip {
                su = src - 1 - su;
            }
            let ch = rows[sv][su];
            if ch == '.' {
                continue;
            }
            if let Some(&(_, col)) = sp.colors.iter().find(|(c, _)| *c == ch) {
                cv.block(x0 + u as i32 * k, y0 + v as i32 * k, k, if dark { shade(col, 0.7) } else { col });
            }
        }
    }
}

fn bomb(a: i32) -> &'static [&'static str] {
    match a {
        8 => &[".....ss.", "....f...", "..kkkf..", ".kwkkkk.", "kwkkkkkk", "kkkkkkkk", ".kkkkkk.", "..kkkk.."],
        6 => &["....s.", "...f..", ".kkkk.", "kwkkkk", "kkkkkk", ".kkkk."],
        _ => &["..fs", ".kk.", "kwkk", ".kk."],
    }
}

fn hero(a: i32, facing: Dir, step: bool) -> Vec<&'static str> {
    match a {
        8 => {
            // single-pixel eyes, like the 6 art: bigger ones read as googly
            let eyes = match facing {
                Dir::Down => "cceccecc",
                Dir::Up => "cccccccc",
                Dir::Left => "cecceccc",
                Dir::Right => "ccceccec",
            };
            vec!["..cccc..", ".cccccc.", eyes, "cccccccc", "cccccccc", ".dccccd.", "..dddd..", if step { "kk...kk." } else { ".kk..kk." }]
        }
        6 => {
            let eyes = match facing {
                Dir::Down => "ceccec",
                Dir::Up => "cccccc",
                Dir::Left => "eccecc",
                Dir::Right => "ccecce",
            };
            vec![".cccc.", eyes, "cccccc", "dccccd", ".dddd.", if step { "k...k." } else { ".k..k." }]
        }
        _ => {
            let eyes = match facing {
                Dir::Down => "ceec",
                Dir::Up => "cccc",
                Dir::Left => "eecc",
                Dir::Right => "ccee",
            };
            vec![".cc.", eyes, "dccd", if step { "k..k" } else { ".kk." }]
        }
    }
}

fn item_frame(item: Item) -> Rgb {
    match item {
        Item::BombUp => [70, 100, 215],
        Item::FireUp | Item::FullFire => [215, 70, 45],
        Item::SpeedUp => [45, 170, 95],
        Item::SpeedDown => [130, 95, 60],
        Item::Kick | Item::Punch | Item::Glove | Item::LineBomb => [150, 80, 200],
        Item::Remote | Item::Pierce | Item::Rubber | Item::Mine => [90, 90, 110],
        Item::BombPass | Item::BlockPass => [40, 160, 170],
        Item::Heart => [250, 200, 215],
        Item::Vest => [230, 110, 160],
        Item::Skull => [60, 30, 80],
    }
}

/// The icon inside an item's frame, 6x6 for the 8 art and 4x4 for the 6 art; the 4 art
/// takes the middle 2x2 of the 4x4.
fn item_icon6(item: Item) -> [&'static str; 6] {
    match item {
        Item::BombUp => ["....s.", "...f..", ".kkkk.", "kwkkkk", "kkkkkk", ".kkkk."],
        Item::FireUp => ["..y...", "..yy..", ".yyry.", "yyrrry", "yrrrry", ".rrrr."],
        Item::FullFire => [".y..y.", ".yy.yy", "yyryyy", "yrrrry", "yrrrry", ".rrrr."],
        Item::SpeedUp => ["w..w..", ".w..w.", "..w..w", "..w..w", ".w..w.", "w..w.."],
        Item::SpeedDown => ["......", "bbbbbb", "bbbbbb", ".k..k.", ".k..k.", "......"],
        Item::Kick => ["..ww..", "..ww..", "..ww..", "..wwww", ".wwwww", "......"],
        Item::Punch => [".wwww.", "wwwwww", "wwwwww", "wwwww.", ".www..", "......"],
        Item::Glove => ["w.w.w.", "w.w.w.", "wwwww.", "wwwwww", ".wwww.", "..ww.."],
        Item::LineBomb => [".kk.kk", ".kk.kk", "......", "rrrrr.", "rrrrrr", "rrrrr."],
        Item::Remote => ["....y.", "...g..", ".kkgk.", ".krkk.", ".kkkk.", ".kkkk."],
        Item::Pierce => ["..kk..", ".kkkk.", "wwwwww", ".kkkk.", "..kk..", "......"],
        Item::Rubber => ["..pp..", ".pppp.", "ppwppp", "pppppp", ".pppp.", "..pp.."],
        Item::Mine => ["......", "..r...", "..g...", ".gggg.", "gggggg", "......"],
        Item::BombPass => ["..gg..", ".gggg.", "gwgggg", "gggggg", ".gggg.", "..gg.."],
        Item::BlockPass => ["bbgbbb", "gggggg", "bbbbgb", "gggggg", "bgbbbb", "gggggg"],
        Item::Heart => [".h..h.", "hhhhhh", "hwhhhh", ".hhhh.", "..hh..", "......"],
        Item::Vest => ["cc..cc", "cccccc", ".cccc.", ".cccc.", ".cccc.", "......"],
        Item::Skull => [".wwww.", "wkwwkw", "wwwwww", ".wkwk.", ".wwww.", "......"],
    }
}

fn item_icon4(item: Item) -> [&'static str; 4] {
    match item {
        Item::BombUp => [".kk.", "kwkk", "kkkk", ".kk."],
        Item::FireUp => ["..y.", ".yy.", "yyry", ".rr."],
        Item::FullFire => ["y.y.", "yyyy", "yrry", ".rr."],
        Item::SpeedUp => ["w.w.", ".w.w", ".w.w", "w.w."],
        Item::SpeedDown => ["....", "bbbb", "bbbb", "k..k"],
        Item::Kick => [".ww.", ".ww.", ".www", ".www"],
        Item::Punch => [".ww.", "wwww", "wwww", ".ww."],
        Item::Glove => ["w.w.", "wwww", "wwww", ".ww."],
        Item::LineBomb => ["k.k.", "k.k.", "rrrr", "...."],
        Item::Remote => ["..y.", ".kg.", "krkk", "kkkk"],
        Item::Pierce => [".kk.", "wwww", "wwww", ".kk."],
        Item::Rubber => [".pp.", "pwpp", "pppp", ".pp."],
        Item::Mine => [".r..", ".g..", "gggg", "gggg"],
        Item::BombPass => [".gg.", "gwgg", "gggg", ".gg."],
        Item::BlockPass => ["bgbb", "gggg", "bbgb", "gggg"],
        Item::Heart => ["h..h", "hhhh", "hhhh", ".hh."],
        Item::Vest => ["c..c", "cccc", ".cc.", ".cc."],
        Item::Skull => ["wwww", "kwwk", "wwww", "w.w."],
    }
}

fn item_pal(frame: Rgb) -> impl Fn(char) -> Option<Rgb> {
    move |ch| match ch {
        'o' => Some(frame),
        'i' => Some([28, 28, 40]),
        'k' => Some([20, 20, 28]),
        'w' => Some([235, 235, 245]),
        'y' => Some([255, 225, 70]),
        'r' => Some([255, 120, 35]),
        'f' => Some([150, 120, 85]),
        's' => Some([255, 225, 70]),
        'g' => Some([150, 150, 165]),
        'b' => Some([170, 110, 60]),
        'p' => Some([255, 120, 170]),
        'c' => Some([90, 200, 230]),
        'h' => Some([235, 45, 70]),
        _ => None,
    }
}

fn draw_item(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, item: Item) {
    let s = a * k;
    let pal = item_pal(item_frame(item));
    cv.block(x0, y0, s, item_frame(item));
    cv.block(x0 + k, y0 + k, s - 2 * k, [28, 28, 40]);
    match a {
        8 => cv.sprite(x0 + k, y0 + k, k, &item_icon6(item), &pal),
        6 => cv.sprite(x0 + k, y0 + k, k, &item_icon4(item), &pal),
        _ => {
            let i4 = item_icon4(item);
            let mid: Vec<String> = i4[1..3].iter().map(|r| r[1..3].to_string()).collect();
            let rows: Vec<&str> = mid.iter().map(String::as_str).collect();
            cv.sprite(x0 + k, y0 + k, k, &rows, &pal);
        }
    }
}

/// Draw a bomb with its top-left at (x0, y0).
fn draw_bomb(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, b: &Bomb, t: u64) {
    // the body throbs red faster and faster as the fuse runs down
    let waiting = matches!(b.kind, BombKind::Remote) || (b.kind == BombKind::Mine && b.fuse > 20);
    let urgent = b.fuse < FUSE / 3;
    let period = if urgent { 6 } else { 16 };
    let hot = !waiting && (t / period) % 2 == 0 && b.fuse < FUSE * 2 / 3;
    let base = match b.kind {
        BombKind::Pierce => [30, 40, 90],
        BombKind::Rubber => [120, 40, 90],
        BombKind::Mine => [70, 70, 78],
        _ => [24, 24, 34],
    };
    let body = if hot { [110, 22, 26] } else { base };
    // remote bombs wear a blinking red light instead of a burning fuse
    let spark = if b.kind == BombKind::Remote {
        if (t / 20) % 2 == 0 { [255, 40, 40] } else { [90, 20, 20] }
    } else if (t / 3) % 2 == 0 {
        [255, 235, 90]
    } else {
        [255, 120, 40]
    };
    cv.sprite(x0, y0, k, bomb(a), |c| match c {
        'k' => Some(body),
        'w' => Some([210, 210, 230]),
        'f' => Some([150, 120, 85]),
        's' => Some(spark),
        _ => None,
    });
}

/// Draw the part of the arena the camera sees into `area`. `cam` is the world point
/// (units) at the middle of the view; an arena smaller than the view is centred.
/// `shake` nudges everything sideways a cell.
pub fn draw_arena(g: &Game, zoom: Zoom, cam: (i32, i32), shake: i16, area: Rect, buf: &mut Buffer) {
    if zoom == Zoom::SMALL {
        draw_small(g, cam, area, buf);
        return;
    }
    let s = zoom.px();
    let (a, k) = zoom.art();
    let (vw, vh) = (area.width as i32, area.height as i32 * 2);
    let (aw, ah) = (s * g.w as i32, s * g.h as i32);
    // the window's top-left in world pixels
    let place = |c: i32, view: i32, world: i32| if world <= view { (world - view) / 2 } else { (c * s / TILE - view / 2).clamp(0, world - view) };
    let (ox, oy) = (place(cam.0, vw, aw) - shake as i32, place(cam.1, vh, ah) & !1);
    let mut cv = Canvas::window(vw, vh, ox, oy, [20, 20, 26]);
    let t = g.steps;
    // units to the top-left pixel of a tile-sized sprite centred there, on whole art pixels
    let snap = |v: i32| (v * s / TILE - s / 2).div_euclid(k) * k;
    // only the tiles in view (and one round them, for things that overhang)
    let tx0 = (ox / s - 1).max(0) as usize;
    let ty0 = (oy / s - 1).max(0) as usize;
    let tx1 = (((ox + vw) / s + 2).max(0) as usize).min(g.w);
    let ty1 = (((oy + vh) / s + 2).max(0) as usize).min(g.h);
    let (gw, gh) = (g.w, g.h);

    for ty in ty0..ty1 {
        for tx in tx0..tx1 {
            let (x0, y0) = (tx as i32 * s, ty as i32 * s);
            let floor = if (tx + ty) % 2 == 0 { FLOOR_A } else { FLOOR_B };
            cv.block(x0, y0, s, floor);
            match g.tiles[ty][tx] {
                Tile::Wall if g.ground[ty][tx] == Ground::Water => water(&mut cv, x0, y0, a, k, t),
                Tile::Wall if g.ground[ty][tx] == Ground::Tree => canopy(&mut cv, x0, y0, a, k, tx, ty),
                Tile::Wall => {
                    let edge = tx == 0 || ty == 0 || tx == gw - 1 || ty == gh - 1;
                    let (h, m, d) = if edge {
                        ([118, 118, 136], [92, 92, 110], [58, 58, 72])
                    } else {
                        ([205, 205, 218], [152, 152, 170], [92, 92, 108])
                    };
                    cv.sprite(x0, y0, k, wall(a), |c| match c {
                        'h' => Some(h),
                        'g' => Some(m),
                        'd' => Some(d),
                        _ => None,
                    });
                }
                Tile::Pressure(_) => {}
                Tile::Soft => soft_block(&mut cv, x0, y0, a, k, None, t, g.theme),
                Tile::Burning(left) => soft_block(&mut cv, x0, y0, a, k, Some(left), t, g.theme),
                Tile::Dud => {
                    cv.sprite(x0, y0, k, dud(a), |c| match c {
                        'y' => Some([250, 205, 60]),
                        'r' => Some([215, 50, 55]),
                        'w' => Some([245, 240, 230]),
                        'b' => Some([70, 40, 60]),
                        'f' => Some([150, 120, 85]),
                        _ => None,
                    });
                }
                Tile::Floor => {
                    if g.theme == Theme::Orchard && g.ground[ty][tx] == Ground::Plain {
                        // a few meadow flowers in the grass
                        let h = hash(tx as i32, ty as i32, 11);
                        if h % 3 == 0 {
                            let (fx, fy) = ((h / 3 % (a as u32 - 2)) as i32 + 1, (h / 17 % (a as u32 - 2)) as i32 + 1);
                            let petal = [[250, 250, 245], [250, 220, 90], [235, 150, 200]][(h / 91 % 3) as usize];
                            cv.block(x0 + fx * k, y0 + fy * k, k, petal);
                        }
                    }
                    if g.exit == Some((tx, ty)) {
                        exit_hatch(&mut cv, x0, y0, a, k, g.exit_open, t);
                    }
                    ground(&mut cv, x0, y0, a, k, g.ground[ty][tx], t);
                    // the block above casts a shadow onto the top of this floor tile
                    if ty > 0 && g.tiles[ty - 1][tx] != Tile::Floor {
                        for x in 0..s {
                            for y in 0..k {
                                cv.set(x0 + x, y0 + y, FLOOR_SHADOW);
                            }
                        }
                    }
                    if let Some(item) = g.items[ty][tx] {
                        draw_item(&mut cv, x0, y0, a, k, item);
                    }
                }
            }
        }
    }

    // grounded bombs; armed mines are buried out of sight
    for b in g.bombs.iter().filter(|b| b.grounded() && !b.armed) {
        draw_bomb(&mut cv, snap(b.x), snap(b.y), a, k, b, t);
    }

    for ty in ty0..ty1 {
        for tx in tx0..tx1 {
            if let Some(f) = g.flames[ty][tx] {
                flame(&mut cv, tx as i32 * s, ty as i32 * s, a, k, f, t);
            }
        }
    }

    // pressure blocks fall in from above, over whatever is next to them
    for ty in ty0..ty1 {
        for tx in tx0..tx1 {
            if let Tile::Pressure(f) = g.tiles[ty][tx] {
                let lift = (DROP.saturating_sub(f) as i32 * s / DROP as i32).div_euclid(k) * k;
                let (x0, y0) = (tx as i32 * s, ty as i32 * s - lift);
                cv.sprite(x0, y0, k, wall(a), |c| match c {
                    'h' => Some([235, 190, 60]),
                    'g' => Some(if (tx + ty) % 2 == 0 { [190, 140, 30] } else { [170, 120, 25] }),
                    'd' => Some([90, 62, 18]),
                    _ => None,
                });
            }
        }
    }

    // eggs left by tamed baddies, wobbling
    for e in &g.eggs {
        let wob = if (t / 30) % 2 == 0 { 0 } else { k };
        draw_egg(&mut cv, snap(e.x) + wob, snap(e.y), a, k, egg_spot(&g.species[e.species]));
    }

    // baddies
    for b in g.baddies.iter().filter(|b| b.alive) {
        let sp = &g.species[b.species];
        let (x0, mut y0) = (snap(b.x), snap(b.y));
        if b.disguised {
            soft_block(&mut cv, x0, y0, a, k, None, t, g.theme);
            continue;
        }
        if let State::Hop { t: ht, dur, .. } = b.state {
            let arc = (std::f32::consts::PI * ht as f32 / dur as f32).sin();
            cv.block(x0 + (a / 2 - 1) * k, y0 + (a - 1) * k, 2 * k, [30, 70, 35]);
            y0 -= ((arc * s as f32 * 0.8) as i32).div_euclid(k) * k;
        }
        // flashing after a hit
        if b.hurt > 0 && (t / 3) % 2 == 0 {
            continue;
        }
        let frame = if !sp.art2.is_empty() && (b.walk / 8) % 2 == 1 { &sp.art2 } else { &sp.art };
        let turns = match b.state {
            State::Roll => (b.walk / 4) % 4,
            _ => 0,
        };
        let dark = matches!(b.state, State::Curl(_) | State::Roll);
        draw_species(&mut cv, x0, y0, a, k, sp, frame, turns as usize, dark, b.dir == Dir::Left);
    }

    // players, back to front
    let mut order: Vec<usize> = (0..g.players.len()).filter(|&i| g.players[i].joined && g.players[i].rider.is_none()).collect();
    order.sort_by_key(|&i| g.players[i].y);
    for i in order {
        let p = &g.players[i];
        let (x0, y0) = (snap(p.x), snap(p.y));
        if !p.alive && (p.out || p.respawn <= 90) {
            continue;
        }
        // invulnerable after a saved hit: blink
        if p.alive && p.invuln > 0 && (t / 4) % 2 == 0 {
            continue;
        }
        // invisible: only a shimmer now and then
        if p.disease == Some(Disease::Invisible) && (t / 3) % 12 != 0 {
            continue;
        }
        // hopping on a mount: up in an arc, with a shadow below
        let mut lift = 0;
        if let Some((_, _, ht, dur)) = p.hop {
            let arc = (std::f32::consts::PI * ht as f32 / dur as f32).sin();
            lift = ((arc * s as f32 * 0.8) as i32).div_euclid(k) * k;
            cv.block(x0 + (a / 2 - 1) * k, y0 + (a - 1) * k, 2 * k, [30, 70, 35]);
        }
        // eggs carried for later trail behind
        for (n, &sp) in p.spare.iter().enumerate() {
            let (dx, dy) = p.facing.vec();
            let off = (n as i32 + 1) * s * 2 / 3;
            draw_egg(&mut cv, x0 - dx * off, y0 - dy * off, a, k, egg_spot(&g.species[sp]));
        }
        if let Some(m) = p.mount {
            let sp = &g.species[m];
            let frame = if !sp.art2.is_empty() && (p.walk / 6) % 2 == 1 { &sp.art2 } else { &sp.art };
            // the rider sits up on its back, the mount's head in front
            draw_hero(&mut cv, x0, y0 - lift - (a / 2) * k, a, k, i, p, t);
            draw_species(&mut cv, x0, y0 - lift, a, k, sp, frame, 0, false, p.facing == Dir::Left);
        } else {
            draw_hero(&mut cv, x0, y0 - lift, a, k, i, p, t);
        }
        if p.alive && p.stun > 0 {
            // dizzy: two stars circling the head
            let (sx, sy) = if (t / 6) % 2 == 0 { (0, 0) } else { (a - 2, 0) };
            cv.block(x0 + sx * k, y0 + sy * k - k, k, [255, 230, 80]);
            cv.block(x0 + (a - 2 - sx) * k, y0 - k, k, [255, 255, 255]);
        }
    }

    // bombs in the air and in hands: above everything on the ground
    for b in g.bombs.iter().filter(|b| !b.grounded()) {
        let (x0, mut y0) = (snap(b.x), snap(b.y));
        match b.motion {
            Motion::Fly { t: ft, dur, .. } => {
                let arc = (std::f32::consts::PI * ft as f32 / dur as f32).sin();
                let lift = ((arc * s as f32 * 0.9) as i32).div_euclid(k) * k;
                // a shadow on the ground under it
                cv.block(x0 + (a / 2 - 1) * k, y0 + (a - 1) * k, 2 * k, [30, 70, 35]);
                y0 -= lift;
            }
            Motion::Carried { .. } => y0 -= (a * 2 / 3) * k,
            _ => {}
        }
        draw_bomb(&mut cv, x0, y0, a, k, b, t);
    }

    // firework sparks
    let hues: [[Rgb; 2]; 6] = [
        [[255, 90, 90], [255, 210, 120]],
        [[110, 200, 255], [230, 245, 255]],
        [[255, 215, 70], [255, 250, 200]],
        [[150, 255, 130], [235, 255, 210]],
        [[230, 120, 255], [255, 220, 255]],
        [[255, 150, 60], [255, 235, 180]],
    ];
    for sp in &g.sparks {
        let c = hues[sp.hue as usize % 6][((sp.life / 4) % 2) as usize];
        let c = if sp.life < 12 { shade(c, sp.life as f32 / 12.0) } else { c };
        let (x, y) = ((sp.x * s / TILE).div_euclid(k) * k, (sp.y * s / TILE).div_euclid(k) * k);
        cv.block(x, y, k, c);
    }

    // revenge riders on the border, each in a little cart
    for (i, p) in g.players.iter().enumerate() {
        let Some(pos) = p.rider else { continue };
        let (ux, uy) = g.rider_pos(pos);
        let (x0, y0) = (snap(ux), snap(uy));
        draw_hero(&mut cv, x0, y0 - k, a, k, i, p, t);
        let cart = [60, 60, 70];
        cv.block(x0, y0 + (a - 3) * k, k, cart);
        for x in 0..a {
            for y in a - 3..a - 1 {
                cv.block(x0 + x * k, y0 + y * k, k, cart);
            }
        }
        cv.block(x0 + k, y0 + (a - 1) * k, k, [20, 20, 28]);
        cv.block(x0 + (a - 2) * k, y0 + (a - 1) * k, k, [20, 20, 28]);
    }

    blit(&cv, area.x as i32, area.y as i32, area, buf);
}

/// Two pixel rows per cell.
fn blit(cv: &Canvas, ox: i32, oy: i32, area: Rect, buf: &mut Buffer) {
    for cy in 0..(cv.h / 2) {
        for cx in 0..cv.w {
            let (x, y) = (ox + cx, oy + cy);
            if x < area.x as i32 || y < area.y as i32 || x >= area.right() as i32 || y >= area.bottom() as i32 {
                continue;
            }
            let top = cv.px[(cy * 2 * cv.w + cx) as usize];
            let bot = cv.px[((cy * 2 + 1) * cv.w + cx) as usize];
            let cell = &mut buf[(x as u16, y as u16)];
            cell.set_symbol("▀");
            cell.set_fg(Color::Rgb(top[0], top[1], top[2]));
            cell.set_bg(Color::Rgb(bot[0], bot[1], bot[2]));
        }
    }
}

/// Stage ground under a floor tile: belts, arrows, warps, ice.
fn ground(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, gr: Ground, t: u64) {
    // art pixel (u, v) in a frame where the direction `d` points right
    let rotate = |d: Dir, u: i32, v: i32| -> (i32, i32) {
        match d {
            Dir::Right => (u, v),
            Dir::Left => (a - 1 - u, a - 1 - v),
            Dir::Down => (a - 1 - v, u),
            Dir::Up => (v, a - 1 - u),
        }
    };
    match gr {
        Ground::Plain | Ground::Water | Ground::Tree => {}
        Ground::Belt(d) => {
            let shift = (t / 5) as i32;
            for v in 0..a {
                for u in 0..a {
                    let rail = v == 0 || v == a - 1;
                    // chevrons marching the way the belt runs
                    let chevron = (u - (v - a / 2).abs() - shift).rem_euclid(3) == 0;
                    let c = if rail {
                        [44, 44, 52]
                    } else if chevron {
                        [150, 150, 160]
                    } else {
                        [78, 78, 88]
                    };
                    let (x, y) = rotate(d, u, v);
                    cv.block(x0 + x * k, y0 + y * k, k, c);
                }
            }
        }
        Ground::Arrow(d) => {
            for v in 0..a {
                for u in 0..a {
                    let mid = (v * 2 + 1 - a).abs();
                    let head = u >= a / 2 && mid <= (a - 1 - u) * 2 - 1;
                    let shaft = u >= 1 && u < a / 2 && mid <= 1;
                    if head || shaft {
                        let (x, y) = rotate(d, u, v);
                        cv.block(x0 + x * k, y0 + y * k, k, [240, 210, 70]);
                    }
                }
            }
        }
        Ground::Warp => {
            let c = (a as f32 - 1.0) / 2.0;
            let spin = t as f32 / 8.0;
            for v in 0..a {
                for u in 0..a {
                    let (dx, dy) = (u as f32 - c, v as f32 - c);
                    let r = dx.hypot(dy);
                    if r > c + 0.5 {
                        continue;
                    }
                    let ang = dy.atan2(dx) + spin - r * 0.8;
                    let bright = (ang * 2.0).sin() > 0.3;
                    let col = if r < 0.9 {
                        [20, 10, 40]
                    } else if bright {
                        [120, 230, 255]
                    } else {
                        [90, 40, 160]
                    };
                    cv.block(x0 + u * k, y0 + v * k, k, col);
                }
            }
        }
        Ground::Ice => {
            for v in 0..a {
                for u in 0..a {
                    let (x, y) = (x0 + u * k, y0 + v * k);
                    let glint = hash(x, y, 3) % 23 == 0 && (t / 10 + hash(x, y, 5) as u64) % 6 == 0;
                    let c = if glint {
                        [255, 255, 255]
                    } else if (u + v) % 4 == 0 {
                        [190, 228, 245]
                    } else {
                        [168, 214, 238]
                    };
                    cv.block(x, y, k, c);
                }
            }
        }
    }
}

/// A 6x6-pixel item badge (6x3 cells) for the HUD, top-left at cell (x, y).
pub fn item_badge(buf: &mut Buffer, x: u16, y: u16, item: Item, area: Rect) {
    let mut cv = Canvas::new(6, 6);
    draw_item(&mut cv, 0, 0, 6, 1, item);
    blit(&cv, x as i32, y as i32, area, buf);
}

/// A player's hero in their colour, 8x8 pixels (8x4 cells), for the HUD.
pub fn hero_badge(buf: &mut Buffer, x: u16, y: u16, i: usize, p: &Player, t: u64, area: Rect) {
    let mut cv = Canvas::new(8, 8);
    cv.block(0, 0, 8, [24, 24, 30]);
    draw_hero(&mut cv, 0, 0, 8, 1, i, p, t);
    blit(&cv, x as i32, y as i32, area, buf);
}

#[allow(clippy::too_many_arguments)]
fn draw_hero(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, i: usize, p: &Player, t: u64) {
    let base = PLAYER[i];
    let (mut c, mut d) = (base, shade(base, 0.6));
    if p.rider.is_some() {
        (c, d) = (shade(base, 0.75), shade(base, 0.5));
    } else if !p.alive {
        let flash = if (t / 3) % 2 == 0 { [255, 255, 255] } else { [255, 60, 40] };
        (c, d) = (flash, flash);
    } else if p.disease.is_some() {
        // sick: the body pulses toward a sickly violet
        let f = 0.5 + 0.5 * ((t as f32) / 8.0).sin();
        (c, d) = (mix(base, [120, 60, 150], f * 0.7), mix(shade(base, 0.6), [70, 30, 90], f * 0.7));
    } else if p.vest > 0 {
        // the vest: a white shimmer running through
        let f = 0.5 + 0.5 * ((t as f32) / 3.0).sin();
        c = mix(base, [255, 255, 255], f * 0.6);
    }
    let step = p.walk > 0 && (p.walk / 6) % 2 == 1;
    cv.sprite(x0, y0, k, &hero(a, p.facing, step), |ch| match ch {
        'c' => Some(c),
        'd' => Some(d),
        'e' => Some([255, 255, 255]),
        'p' | 'k' => Some([20, 20, 28]),
        _ => None,
    });
}

#[allow(clippy::too_many_arguments)]
fn soft_block(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, burning: Option<u32>, t: u64, theme: Theme) {
    // burning: the block glows, then crumbles away art pixel by art pixel
    let progress = burning.map_or(0.0, |left| 1.0 - left as f32 / crate::game::BURN as f32);
    let rows = match theme {
        Theme::Brick => soft(a),
        Theme::Orchard => hedge(a),
    };
    for (dy, row) in rows.iter().enumerate() {
        for (dx, ch) in row.chars().enumerate() {
            let (x, y) = (x0 + dx as i32 * k, y0 + dy as i32 * k);
            let mut c = match (theme, ch) {
                (Theme::Brick, 'l') => [222, 160, 92],
                (Theme::Brick, 'b') => [188, 116, 58],
                (Theme::Brick, 'r') => [108, 58, 32],
                (Theme::Orchard, 'l') => [125, 195, 85],
                (Theme::Orchard, 'g') => [75, 150, 60],
                (Theme::Orchard, 'd') => [40, 100, 45],
                (Theme::Orchard, 'r') => [215, 45, 45],
                _ => continue,
            };
            if burning.is_some() {
                if (hash(x, y, 7) % 1000) as f32 / 1000.0 < progress * 1.2 - 0.2 {
                    continue;
                }
                let glow = if (hash(x, y, t / 2) % 3) == 0 { [255, 200, 60] } else { [235, 90, 30] };
                c = mix(c, glow, (0.4 + progress).min(1.0));
            }
            cv.block(x, y, k, c);
        }
    }
}

/// A flame tile: a hot core disc plus half-arms toward each linked neighbour,
/// thinning as it burns out. Worked out per art pixel (`a` across), drawn `k` x `k`.
fn flame(cv: &mut Canvas, x0: i32, y0: i32, a: i32, k: i32, f: Flame, t: u64) {
    let life = f.left as f32 / crate::game::FLAME as f32;
    let half = a as f32 / 2.0;
    let r = half * (0.35 + 0.65 * life.sqrt());
    for y in 0..a {
        for x in 0..a {
            let (fx, fy) = (x as f32 + 0.5 - half, y as f32 + 0.5 - half);
            // distance to the core, and to each linked arm's centre line
            let mut d = fx.hypot(fy);
            if f.links & LEFT != 0 && fx <= 0.0 || f.links & RIGHT != 0 && fx >= 0.0 {
                d = d.min(fy.abs());
            }
            if f.links & UP != 0 && fy <= 0.0 || f.links & DOWN != 0 && fy >= 0.0 {
                d = d.min(fx.abs());
            }
            let (px, py) = (x0 + x * k, y0 + y * k);
            let jitter = (hash(px, py, t / 2) % 100) as f32 / 100.0 * 0.24;
            let n = d / r + jitter;
            if n > 1.0 {
                continue;
            }
            let c = if n < 0.4 {
                [255, 250, 210]
            } else if n < 0.75 {
                [255, 205, 50]
            } else {
                [240, 92, 24]
            };
            cv.block(px, py, k, c);
        }
    }
}

/// Two-letter labels for the glyph size.
fn item_label(item: Item) -> &'static str {
    match item {
        Item::BombUp => "B+",
        Item::FireUp => "F+",
        Item::FullFire => "FF",
        Item::SpeedUp => "S+",
        Item::SpeedDown => "S-",
        Item::Kick => "Ki",
        Item::Punch => "Pu",
        Item::Glove => "Gl",
        Item::LineBomb => "Li",
        Item::Remote => "Re",
        Item::Pierce => "Pi",
        Item::Rubber => "Ru",
        Item::Mine => "Mi",
        Item::BombPass => "BP",
        Item::BlockPass => "WP",
        Item::Heart => "<3",
        Item::Vest => "Ve",
        Item::Skull => "%%",
    }
}

/// The smallest size: two glyphs per tile, players snapped to tiles.
fn draw_small(g: &Game, cam: (i32, i32), area: Rect, buf: &mut Buffer) {
    let t = g.steps;
    let (gw, gh) = (g.w, g.h);
    // whole tiles: 2 cells by 1, centred on the camera or on a small arena
    let place = |c: i32, view: i32, world: i32| if world <= view { (world - view) / 2 } else { (c.div_euclid(TILE) - view / 2).clamp(0, world - view) };
    let (vw, vh) = (area.width as i32 / 2, area.height as i32);
    let (tx_off, ty_off) = (place(cam.0, vw, gw as i32), place(cam.1, vh, gh as i32));
    let (ox, oy) = (area.x as i32 - tx_off * 2, area.y as i32 - ty_off);
    let mut put = |tx: i32, ty: i32, text: &str, fg: Rgb, bg: Rgb| {
        for (k, ch) in text.chars().enumerate() {
            let (x, y) = (ox + tx * 2 + k as i32, oy + ty);
            if x < area.x as i32 || y < area.y as i32 || x >= area.right() as i32 || y >= area.bottom() as i32 {
                continue;
            }
            let cell = &mut buf[(x as u16, y as u16)];
            cell.set_char(ch);
            cell.set_fg(Color::Rgb(fg[0], fg[1], fg[2]));
            cell.set_bg(Color::Rgb(bg[0], bg[1], bg[2]));
        }
    };
    let floor_at = |x: i32, y: i32| if (x + y) % 2 == 0 { FLOOR_A } else { FLOOR_B };
    for ty in 0..gh {
        for tx in 0..gw {
            let (x, y) = (tx as i32, ty as i32);
            let floor = floor_at(x, y);
            match g.tiles[ty][tx] {
                Tile::Wall if g.ground[ty][tx] == Ground::Water => put(x, y, "~~", [150, 200, 240], [45, 95, 175]),
                Tile::Wall if g.ground[ty][tx] == Ground::Tree => put(x, y, "♣♣", [110, 175, 80], [30, 80, 35]),
                Tile::Wall => put(x, y, "██", [150, 150, 168], floor),
                Tile::Pressure(_) => put(x, y, "▓▓", [235, 190, 60], [90, 62, 18]),
                Tile::Dud => put(x, y, "!!", [250, 205, 60], [150, 30, 35]),
                Tile::Soft => put(x, y, "▒▒", [205, 130, 66], [108, 58, 32]),
                Tile::Burning(_) => put(x, y, "▒▒", [255, 200, 60], [200, 70, 25]),
                Tile::Floor => match (g.items[ty][tx], g.ground[ty][tx]) {
                    (Some(item), _) => put(x, y, item_label(item), [255, 255, 255], item_frame(item)),
                    (None, Ground::Belt(d) | Ground::Arrow(d)) => {
                        let ch = match d {
                            Dir::Right => ">>",
                            Dir::Left => "<<",
                            Dir::Up => "^^",
                            Dir::Down => "vv",
                        };
                        let belt = matches!(g.ground[ty][tx], Ground::Belt(_));
                        put(x, y, ch, if belt { [150, 150, 160] } else { [240, 210, 70] }, if belt { [70, 70, 80] } else { floor })
                    }
                    (None, Ground::Warp) => put(x, y, "()", [120, 230, 255], [60, 30, 110]),
                    (None, Ground::Ice) => put(x, y, "  ", [255, 255, 255], [175, 218, 240]),
                    (None, Ground::Plain | Ground::Water | Ground::Tree) => put(x, y, "  ", floor, floor),
                },
            }
        }
    }
    for b in g.bombs.iter().filter(|b| !b.armed) {
        let hot = b.fuse < FUSE / 3 && (t / 6) % 2 == 0;
        let (x, y) = (b.x.div_euclid(TILE), b.y.div_euclid(TILE));
        put(x, y, "()", if hot { [255, 60, 40] } else { [20, 20, 28] }, floor_at(x, y));
    }
    for ty in 0..gh {
        for tx in 0..gw {
            if let Some(f) = g.flames[ty][tx] {
                let text = if f.links & (LEFT | RIGHT) != 0 && f.links & (UP | DOWN) == 0 {
                    "══"
                } else if f.links & (UP | DOWN) != 0 && f.links & (LEFT | RIGHT) == 0 {
                    "║║"
                } else {
                    "╬╬"
                };
                put(tx as i32, ty as i32, text, [255, 245, 170], [240, 110, 25]);
            }
        }
    }
    for (i, p) in g.players.iter().enumerate() {
        if let Some(pos) = p.rider {
            let (ux, uy) = g.rider_pos(pos);
            put(ux.div_euclid(TILE), uy.div_euclid(TILE), &format!("&{}", i + 1), [20, 20, 28], shade(PLAYER[i], 0.7));
            continue;
        }
        if !p.joined || !(p.alive || (!p.out && p.respawn > 90 && (t / 3) % 2 == 0)) {
            continue;
        }
        let (tx, ty) = p.tile();
        put(tx as i32, ty as i32, &format!("@{}", i + 1), [20, 20, 28], PLAYER[i]);
    }
}

/// Where the view looks and how close. Follows the players: the middle of everyone
/// still standing, zoomed in as far as keeps them all in view (with room round
/// them), out again when they spread. `+`/`-` fix a size by hand, `0` goes back to auto.
pub struct Camera {
    /// The world point (units) at the middle of the view.
    pub pos: (f32, f32),
    pub zoom: Zoom,
    pub manual: Option<Zoom>,
}

impl Camera {
    pub fn new() -> Camera {
        Camera { pos: (0.0, 0.0), zoom: Zoom(8), manual: None }
    }

    /// Who the camera keeps in view: players in the round, or the whole arena.
    fn targets(g: &Game) -> Option<(i32, i32, i32, i32)> {
        let mut bb: Option<(i32, i32, i32, i32)> = None;
        for p in g.players.iter().filter(|p| p.joined && p.alive && p.rider.is_none()) {
            bb = Some(match bb {
                None => (p.x, p.y, p.x, p.y),
                Some((a, b, c, d)) => (a.min(p.x), b.min(p.y), c.max(p.x), d.max(p.y)),
            });
        }
        bb
    }

    /// Move toward the players. `snap` jumps straight there (a new round).
    pub fn update(&mut self, g: &Game, view: Rect, dt: f32, snap: bool) {
        let bb = Camera::targets(g);
        let (bw, bh) = bb.map_or((g.w as f32, g.h as f32), |(a, b, c, d)| ((c - a) as f32 / TILE as f32, (d - b) as f32 / TILE as f32));
        // enough room round everyone, and never closer than a classic arena's worth
        let need = |z: Zoom, extra: f32| {
            let (tw, th) = z.tiles_in(view.width, view.height);
            tw >= (bw + 5.0 + extra).max((g.w as f32).min(17.0)) && th >= (bh + 4.0 + extra).max((g.h as f32).min(11.0))
        };
        self.zoom = match self.manual {
            Some(z) => z,
            None => {
                let best = Zoom::pixel_sizes().find(|&z| need(z, 0.0)).unwrap_or(Zoom(4));
                // zoom in only with room to spare, so it doesn't flicker at the edge
                if best.px() > self.zoom.px() && !need(best, 2.0) && !snap { self.zoom } else { best }
            }
        };
        if self.zoom != Zoom::SMALL && self.zoom.tiles_in(view.width, view.height).0 < 9.0 {
            self.zoom = Zoom::SMALL;
        }
        let target = bb.map_or(((g.w as i32 * TILE / 2) as f32, (g.h as i32 * TILE / 2) as f32), |(a, b, c, d)| ((a + c) as f32 / 2.0, (b + d) as f32 / 2.0));
        let far = (target.0 - self.pos.0).abs() > 8.0 * TILE as f32 || (target.1 - self.pos.1).abs() > 8.0 * TILE as f32;
        if snap || far {
            self.pos = target;
        } else {
            let f = (dt * 5.0).min(1.0);
            self.pos.0 += (target.0 - self.pos.0) * f;
            self.pos.1 += (target.1 - self.pos.1) * f;
        }
    }

    pub fn at(&self) -> (i32, i32) {
        (self.pos.0 as i32, self.pos.1 as i32)
    }

    pub fn label(&self) -> String {
        match self.manual {
            Some(z) => z.name(),
            None => format!("auto {}", self.zoom.name()),
        }
    }
}

/// A 3x5 pixel font for the top bar (upper case, digits, a little punctuation, and the
/// lower-case u and o of the logo).
fn glyph(c: char) -> [&'static str; 5] {
    match c.to_ascii_uppercase() {
        _ if c == 'u' => ["...", "...", "#.#", "#.#", "###"],
        _ if c == 'o' => ["...", "...", "###", "#.#", "###"],
        'A' => [".#.", "#.#", "###", "#.#", "#.#"],
        'B' => ["##.", "#.#", "##.", "#.#", "##."],
        'C' => [".##", "#..", "#..", "#..", ".##"],
        'D' => ["##.", "#.#", "#.#", "#.#", "##."],
        'E' => ["###", "#..", "##.", "#..", "###"],
        'F' => ["###", "#..", "##.", "#..", "#.."],
        'G' => [".##", "#..", "#.#", "#.#", ".##"],
        'H' => ["#.#", "#.#", "###", "#.#", "#.#"],
        'I' => ["###", ".#.", ".#.", ".#.", "###"],
        'J' => ["..#", "..#", "..#", "#.#", ".#."],
        'K' => ["#.#", "#.#", "##.", "#.#", "#.#"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'M' => ["#.#", "###", "###", "#.#", "#.#"],
        'N' => ["##.", "#.#", "#.#", "#.#", "#.#"],
        'O' => [".#.", "#.#", "#.#", "#.#", ".#."],
        'P' => ["##.", "#.#", "##.", "#..", "#.."],
        'Q' => [".#.", "#.#", "#.#", "##.", ".##"],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'S' => [".##", "#..", ".#.", "..#", "##."],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        'V' => ["#.#", "#.#", "#.#", "#.#", ".#."],
        'W' => ["#.#", "#.#", "###", "###", "#.#"],
        'X' => ["#.#", "#.#", ".#.", "#.#", "#.#"],
        'Y' => ["#.#", "#.#", ".#.", ".#.", ".#."],
        'Z' => ["###", "..#", ".#.", "#..", "###"],
        '0' => ["###", "#.#", "#.#", "#.#", "###"],
        '1' => [".#.", "##.", ".#.", ".#.", "###"],
        '2' => ["##.", "..#", ".#.", "#..", "###"],
        '3' => ["##.", "..#", ".#.", "..#", "##."],
        '4' => ["#.#", "#.#", "###", "..#", "..#"],
        '5' => ["###", "#..", "##.", "..#", "##."],
        '6' => [".##", "#..", "###", "#.#", "###"],
        '7' => ["###", "..#", ".#.", ".#.", ".#."],
        '8' => ["###", "#.#", "###", "#.#", "###"],
        '9' => ["###", "#.#", "###", "..#", "##."],
        ':' => ["...", ".#.", "...", ".#.", "..."],
        '/' => ["..#", "..#", ".#.", "#..", "#.."],
        '.' => ["...", "...", "...", "...", ".#."],
        '!' => [".#.", ".#.", ".#.", "...", ".#."],
        '-' => ["...", "...", "###", "...", "..."],
        '>' => ["#..", ".#.", "..#", ".#.", "#.."],
        '<' => ["..#", ".#.", "#..", ".#.", "..#"],
        '*' => ["...", "#.#", ".#.", "#.#", "..."],
        _ => ["...", "...", "...", "...", "..."],
    }
}

/// Width in cells of `text` in the pixel font.
pub fn pixel_width(text: &str) -> u16 {
    text.chars().count() as u16 * 4
}

/// The pixel font at `scale` times the size, drawn over whatever is already on screen
/// (for the title), with a dark outline so it reads on anything. Returns the width.
pub fn pixel_text_big(buf: &mut Buffer, x: u16, y: u16, text: &str, colors: &[Rgb], scale: i32, area: Rect) -> u16 {
    let w = text.chars().count() as i32 * 4 * scale;
    let h = 6 * scale;
    let r = Rect::new(x, y, (w as u16).min(area.right().saturating_sub(x)), ((h / 2) as u16).min(area.bottom().saturating_sub(y)));
    let mut cv = grab(buf, r);
    // outline first, then the letters
    for pass in 0..2 {
        for (n, c) in text.chars().enumerate() {
            let col = colors[n % colors.len()];
            for (gy, row) in glyph(c).iter().enumerate() {
                for (gx, ch) in row.chars().enumerate() {
                    if ch != '#' {
                        continue;
                    }
                    let (px, py) = (n as i32 * 4 * scale + gx as i32 * scale, gy as i32 * scale);
                    if pass == 0 {
                        cv.block(px - 1, py - 1, scale + 2, [10, 10, 14]);
                    } else {
                        cv.block(px, py, scale, col);
                    }
                }
            }
        }
    }
    blit(&cv, r.x as i32, r.y as i32, area, buf);
    w as u16
}

/// Read cells back into pixels (two per cell), to draw over them.
fn grab(buf: &Buffer, r: Rect) -> Canvas {
    let mut cv = Canvas::new(r.width as i32, r.height as i32 * 2);
    let rgb = |c: Color| match c {
        Color::Rgb(r, g, b) => [r, g, b],
        _ => [20, 20, 26],
    };
    for cy in 0..r.height {
        for cx in 0..r.width {
            let cell = &buf[(r.x + cx, r.y + cy)];
            let (top, bot) = if cell.symbol() == "▀" { (rgb(cell.fg), rgb(cell.bg)) } else { (rgb(cell.bg), rgb(cell.bg)) };
            cv.set(cx as i32, cy as i32 * 2, top);
            cv.set(cx as i32, cy as i32 * 2 + 1, bot);
        }
    }
    cv
}

/// Draw `text` in the pixel font, top-left at cell (x, y), three cells tall, on `bg`.
/// Returns the width used.
pub fn pixel_text(buf: &mut Buffer, x: u16, y: u16, text: &str, fg: Rgb, bg: Rgb, area: Rect) -> u16 {
    let w = pixel_width(text) as i32;
    let mut cv = Canvas::window(w, 6, 0, 0, bg);
    for (n, c) in text.chars().enumerate() {
        cv.sprite(n as i32 * 4, 0, 1, &glyph(c), |ch| (ch == '#').then_some(fg));
    }
    blit(&cv, x as i32, y as i32, area, buf);
    w as u16
}

/// Dim everything already drawn in `r` to `f` of its brightness (behind the title).
pub fn dim(buf: &mut Buffer, r: Rect, f: f32) {
    let d = |c: Color| match c {
        Color::Rgb(r, g, b) => {
            let [r, g, b] = shade([r, g, b], f);
            Color::Rgb(r, g, b)
        }
        other => other,
    };
    for y in r.top()..r.bottom() {
        for x in r.left()..r.right() {
            let cell = &mut buf[(x, y)];
            let (fg, bg) = (d(cell.fg), d(cell.bg));
            cell.set_fg(fg);
            cell.set_bg(bg);
        }
    }
}

/// Fill a strip of cells with a flat colour (the top bar's background).
pub fn fill(buf: &mut Buffer, r: Rect, bg: Rgb) {
    for y in r.top()..r.bottom() {
        for x in r.left()..r.right() {
            let cell = &mut buf[(x, y)];
            cell.set_symbol(" ");
            cell.set_bg(Color::Rgb(bg[0], bg[1], bg[2]));
        }
    }
}
