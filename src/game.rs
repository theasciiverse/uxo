//! The arena simulation. Fixed 60 Hz step, integer positions (`TILE` units per
//! tile) and a seeded RNG, so a match replays exactly from its inputs: the base for
//! replays and iroh netplay later.
//!
//! One `Game` is one round. Battle rules (no respawn, revenge riders on the border,
//! sudden death) are switched on by `Game::battle`; with them off it is practice:
//! you respawn and play on.

pub const TILE: i32 = 256;
pub const HZ: u32 = 60;
/// The classic arena size (`Game::new`); stages bring their own.
pub const W: usize = 15;
pub const H: usize = 13;
pub const MAX_PLAYERS: usize = 4;

/// Steps a bomb ticks before it bursts (2.5 s).
pub const FUSE: u32 = 150;
/// Steps a flame burns.
pub const FLAME: u32 = 30;
/// Steps a soft block crumbles after a blast reaches it.
pub const BURN: u32 = 30;
const RESPAWN: u32 = 120;
/// Walking speed in units per step at speed level 0 (about 2.8 tiles/s).
const BASE_SPEED: i32 = 12;
const SPEED_STEP: i32 = 3;
pub const MAX_SPEED: i32 = 4;
pub const MIN_SPEED: i32 = -2;
pub const MAX_BOMBS: u32 = 8;
pub const MAX_FIRE: u32 = 8;
/// Closer than this to the lane centre and blocked ahead, you just stop; further out,
/// corner assist slides you round the pillar into the free lane.
const ASSIST_MIN: i32 = TILE / 16;
const ITEM_CHANCE: u32 = 35;
/// Conveyor belts move players and bombs this many units per step.
const BELT_SPEED: i32 = 6;
/// A kicked bomb's speed, units per step (about 9 tiles/s).
const KICK_SPEED: i32 = 40;
/// Punched and thrown bombs fly this many tiles.
const THROW_TILES: i32 = 3;
const FLY_STEPS_PER_TILE: u32 = 6;
const BOUNCE_STEPS: u32 = 8;
/// Invulnerable after a heart takes a hit, or after a comeback.
pub const INVULN: u32 = 120;
const VEST: u32 = 600;
const DISEASE: u32 = 1200;
const STUN: u32 = 60;
const INFECT_COOL: u32 = 60;
const MINE_ARM: u32 = 40;
const MINE_TRIGGER: u32 = 10;
/// Sudden death: when it starts, and how often a pressure block drops.
pub const HURRY_AT: u32 = 120 * HZ;
const FILL_EVERY: u32 = 30;
/// Steps a pressure block takes to fall into place (drawn only).
pub const DROP: u32 = 10;
/// Fire of the bombs revenge riders lob in.
const RIDER_FIRE: u32 = 2;
/// A surge spreads one tile further every this many steps.
const SURGE_RING: u32 = 4;
/// A fizzled bomb lies live for this long, plus up to `UXO_SPAN` more, then goes
/// off by itself; it wobbles for the last `UXO_WOBBLE` steps.
const UXO_MIN: u32 = 4 * HZ;
const UXO_SPAN: u32 = 8 * HZ;
pub const UXO_WOBBLE: u32 = 50;
/// The blast of one of purged's charges when it is set off.
const CHARGE_FIRE: u32 = 3;
/// Steps of leaning on a block before it moves, so brushing past doesn't shove it.
const PUSH_DELAY: u32 = 10;
/// Steps a pushed block takes to move one tile.
pub const SHOVE_STEPS: u32 = 12;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Floor,
    Wall,
    Soft,
    /// A soft block a blast has reached, crumbling; steps left.
    Burning(u32),
    /// Dropped during sudden death; solid like a wall. Steps since it fell.
    Pressure(u32),
    /// One of purged's charges: solid, and a blast sets it off with a real blast of its
    /// own (adventure "duds" goal).
    Dud,
    /// A pushable block: solid, blasts stop at it and don't break it.
    Block,
    /// A locked door of one of four colours; walking into it with that key opens it.
    Door(u8),
    /// A gate, shut (on `Ground::Gate`); it opens while every switch is held down.
    Gate,
    /// A socket, shut until every part on the stage has been picked up.
    Socket,
}

/// Things lying on the floor that are picked up and kept, never burnt.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pickup {
    Key(u8),
    Part,
}

/// A block on its way from one tile to the next (it already fills `to`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shove {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub dir: Dir,
    pub t: u32,
}

/// What a stage asks of you before the exit opens.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Goal {
    /// Battle and practice: no exit.
    None,
    /// Beat every baddie.
    Clear,
    /// Set off every dud.
    Duds,
    /// Beat the boss (no exit: it ends the stage).
    Boss,
}

/// How the arena looks (soft blocks, mostly).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Brick,
    Orchard,
}

/// An egg a tamed baddie left behind: walk over it to ride.
#[derive(Clone, Copy, Debug)]
pub struct Egg {
    pub x: i32,
    pub y: i32,
    pub species: usize,
}

/// A firework spark (looks only).
#[derive(Clone, Copy, Debug)]
pub struct Spark {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: u32,
    pub hue: u8,
}

/// What a floor tile is made of (stage gimmicks).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ground {
    Plain,
    /// Carries players and bombs.
    Belt(Dir),
    /// Turns sliding bombs.
    Arrow(Dir),
    /// Sends you to the next warp.
    Warp,
    /// You slide until something stops you.
    Ice,
    /// Under a wall: it is a pond (looks only).
    Water,
    /// Under a wall: it is a tree's canopy (looks only).
    Tree,
    /// Water a block was pushed into: floor now.
    Sunk,
    /// Held down by a player, a baddie, a bomb or a block, it opens the gates.
    Switch,
    /// Where a gate stands, shut (`Tile::Gate`) or open (floor).
    Gate,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    BombUp,
    FireUp,
    FullFire,
    SpeedUp,
    SpeedDown,
    Kick,
    Punch,
    Glove,
    LineBomb,
    Remote,
    Pierce,
    Rubber,
    Mine,
    BombPass,
    BlockPass,
    Heart,
    Vest,
    Skull,
    Surge,
}

/// How often each item turns up under a soft block, out of the total.
const DROPS: [(Item, u32); 19] = [
    (Item::BombUp, 14),
    (Item::FireUp, 14),
    (Item::SpeedUp, 10),
    (Item::FullFire, 2),
    (Item::SpeedDown, 3),
    (Item::Kick, 5),
    (Item::Punch, 4),
    (Item::Glove, 4),
    (Item::LineBomb, 3),
    (Item::Remote, 3),
    (Item::Pierce, 3),
    (Item::Rubber, 2),
    (Item::Mine, 2),
    (Item::BombPass, 2),
    (Item::BlockPass, 2),
    (Item::Heart, 3),
    (Item::Vest, 2),
    (Item::Skull, 5),
    (Item::Surge, 2),
];

/// One special bomb type at a time (all your bombs are it); picking up another
/// replaces it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BombKind {
    Normal,
    /// Waits for the owner's action button.
    Remote,
    /// The blast goes through soft blocks.
    Pierce,
    /// Kicked, it bounces back off obstacles instead of stopping.
    Rubber,
    /// Hides in the floor and goes off when someone steps on it.
    Mine,
    /// The blast floods outward on every side, round corners, instead of a cross.
    Surge,
}

/// The skull's curses. They pass on by touch and are cured by any other item.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Disease {
    Slow,
    Fast,
    ShortFuse,
    LongFuse,
    /// Drops bombs whenever it can.
    AutoDrop,
    NoBombs,
    MinFire,
    Reverse,
    /// Can't stop walking.
    NoStop,
    Invisible,
    /// Every bomb you drop fizzles into a UXO.
    Duds,
}

const DISEASES: [Disease; 11] = [
    Disease::Slow,
    Disease::Fast,
    Disease::ShortFuse,
    Disease::LongFuse,
    Disease::AutoDrop,
    Disease::NoBombs,
    Disease::MinFire,
    Disease::Reverse,
    Disease::NoStop,
    Disease::Invisible,
    Disease::Duds,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn vec(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }
    fn bit(self) -> u8 {
        match self {
            Dir::Up => UP,
            Dir::Down => DOWN,
            Dir::Left => LEFT,
            Dir::Right => RIGHT,
        }
    }
    fn back(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

pub const UP: u8 = 1;
pub const DOWN: u8 = 2;
pub const LEFT: u8 = 4;
pub const RIGHT: u8 = 8;

/// One player's wishes for one step. `primary` is where they mean to go; `secondary`
/// (the other axis of a diagonal) is tried when the primary way is blocked, so
/// holding right+down at a junction turns the corner.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Cmd {
    pub primary: Option<Dir>,
    pub secondary: Option<Dir>,
    /// Drop a bomb (a press, not a hold).
    pub bomb: bool,
    /// The action button: punch, pick up or throw, set off remote bombs.
    pub action: bool,
    /// The mount button: your mount's ability (hop, dash).
    pub ride: bool,
    /// A fresh direction press (for menus), not a hold.
    pub tap: Option<Dir>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    Still,
    /// Kicked (or carried by a belt): sliding one way until something stops it.
    Slide { dx: i32, dy: i32, speed: i32 },
    /// Punched, thrown or lobbed: in the air from `from` to `to` (units).
    Fly { from: (i32, i32), to: (i32, i32), t: u32, dur: u32, dx: i32, dy: i32 },
    /// Held over a player's head.
    Carried { by: usize },
}

#[derive(Clone, Debug)]
pub struct Bomb {
    pub id: u32,
    /// Centre, in units.
    pub x: i32,
    pub y: i32,
    pub fuse: u32,
    pub owner: usize,
    pub fire: u32,
    pub kind: BombKind,
    pub motion: Motion,
    /// Mines: buried and waiting for a step.
    pub armed: bool,
    pub age: u32,
    /// Lobbed in by a revenge rider.
    pub from_rider: bool,
    /// Players still standing on the bomb since it landed may walk off it.
    pass: [bool; MAX_PLAYERS],
    /// A UXO: it fizzled, and goes off by itself after this many steps.
    pub uxo: Option<u32>,
    /// A UXO that has been kicked, punched, thrown or carried goes off once it is at rest.
    jolted: bool,
    /// Dropped with the duds disease: it will fizzle.
    dud: bool,
    /// No longer counts as one of its owner's bombs (a UXO, a charge going off).
    loose: bool,
}

impl Bomb {
    pub fn tile(&self) -> (usize, usize) {
        (self.x.div_euclid(TILE).max(0) as usize, self.y.div_euclid(TILE).max(0) as usize)
    }
    /// On the board (still or sliding), not in the air or in someone's hands.
    pub fn grounded(&self) -> bool {
        matches!(self.motion, Motion::Still | Motion::Slide { .. })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Flame {
    pub left: u32,
    /// Which neighbours the flame connects to (UP | DOWN | LEFT | RIGHT).
    pub links: u8,
    /// Whose bomb it was, for revenge comebacks.
    pub owner: usize,
    /// Part of a surge (drawn as one spreading sheet).
    pub surge: bool,
}

/// A surge blast on its way out: the tiles it reached last, spreading a ring at a
/// time until it has gone `left` more tiles.
#[derive(Clone, Debug)]
pub struct Surge {
    pub owner: usize,
    left: u32,
    wait: u32,
    front: Vec<(usize, usize)>,
    seen: Vec<(usize, usize)>,
}

/// How far a surge reaches: shorter than the cross, as it goes every way.
pub fn surge_reach(fire: u32) -> u32 {
    fire.div_ceil(2) + 1
}

#[derive(Clone, Debug)]
pub struct Player {
    pub joined: bool,
    pub x: i32,
    pub y: i32,
    pub alive: bool,
    /// Knocked out of this battle round.
    pub out: bool,
    pub respawn: u32,
    pub bombs: u32,
    pub fire: u32,
    pub speed: i32,
    pub kick: bool,
    pub punch: bool,
    pub glove: bool,
    pub line: bool,
    pub kind: BombKind,
    pub bomb_pass: bool,
    pub block_pass: bool,
    pub heart: bool,
    pub invuln: u32,
    pub vest: u32,
    pub stun: u32,
    pub disease: Option<Disease>,
    pub disease_left: u32,
    infect_cool: u32,
    pub facing: Dir,
    /// Steps spent walking, for the walk animation.
    pub walk: u32,
    pub deaths: u32,
    /// Items picked up this round; scattered back onto the arena when knocked out.
    picked: Vec<Item>,
    /// Revenge rider: position along the border ring, in units.
    pub rider: Option<i32>,
    /// Sliding on ice this way.
    ice: Option<Dir>,
    /// Just arrived through a warp (no warping back until you step off).
    warped: bool,
    /// The tamed baddie you ride (its species), and eggs carried for later.
    pub mount: Option<usize>,
    pub spare: Vec<usize>,
    /// Hopping on your mount: from, to (units), step, length.
    pub hop: Option<((i32, i32), (i32, i32), u32, u32)>,
    /// Dashing on your mount: steps left.
    pub dash: u32,
    /// This round's tally, for the score screens.
    pub kills: u32,
    pub blocks: u32,
    pub items_got: u32,
    pub dropped: u32,
    /// Keys held, by colour; kept through a lost life.
    pub keys: [u8; 4],
    /// Steps spent leaning on a block.
    push: u32,
}

impl Player {
    fn new() -> Player {
        Player {
            joined: false,
            x: 0,
            y: 0,
            alive: false,
            out: false,
            respawn: 0,
            bombs: 1,
            fire: 2,
            speed: 0,
            kick: false,
            punch: false,
            glove: false,
            line: false,
            kind: BombKind::Normal,
            bomb_pass: false,
            block_pass: false,
            heart: false,
            invuln: 0,
            vest: 0,
            stun: 0,
            disease: None,
            disease_left: 0,
            infect_cool: 0,
            facing: Dir::Down,
            walk: 0,
            deaths: 0,
            picked: Vec::new(),
            rider: None,
            ice: None,
            warped: false,
            mount: None,
            spare: Vec::new(),
            hop: None,
            dash: 0,
            kills: 0,
            blocks: 0,
            items_got: 0,
            dropped: 0,
            keys: [0; 4],
            push: 0,
        }
    }
    pub fn tile(&self) -> (usize, usize) {
        ((self.x.div_euclid(TILE)) as usize, (self.y.div_euclid(TILE)) as usize)
    }
    fn walk_speed(&self) -> i32 {
        match self.disease {
            Some(Disease::Slow) => BASE_SPEED / 2,
            Some(Disease::Fast) => BASE_SPEED + SPEED_STEP * (MAX_SPEED + 2),
            _ => BASE_SPEED + SPEED_STEP * self.speed,
        }
    }
}

/// Things that happened during a step, for rumble, sound, juice and the log.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Boom { x: usize, y: usize },
    Death { player: usize, by: Option<usize> },
    /// A heart took the hit.
    Saved { player: usize },
    Pickup { player: usize, item: Item },
    Drop { player: usize },
    Kick { player: usize },
    Throw { player: usize },
    Stunned { player: usize },
    Infect { from: usize, to: usize },
    Comeback { player: usize },
    Hurry,
    Pressure { x: usize, y: usize },
    Warp { player: usize },
    BaddieDown { species: usize, by: usize, x: i32, y: i32 },
    BaddieHit { species: usize },
    DudPopped { x: usize, y: usize, by: usize },
    ExitOpen,
    /// Someone reached the open exit, or the boss fell.
    Cleared,
    Mounted { player: usize },
    MountLost { player: usize },
    Shoved { player: usize },
    DoorOpened { player: usize },
    Gates { open: bool },
    Got { player: usize, what: Pickup },
    SocketOpen,
    /// A bomb fizzled and lies there live.
    Fizzled { x: usize, y: usize },
}

pub struct Game {
    /// Arena size in tiles.
    pub w: usize,
    pub h: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub ground: Vec<Vec<Ground>>,
    pub spawns: [(usize, usize); MAX_PLAYERS],
    /// Percent of soft blocks hiding an item.
    pub item_chance: u32,
    /// When sudden death starts (steps into the round); `u32::MAX` for never.
    pub hurry_at: u32,
    /// Items every player starts the round with.
    pub give: Vec<Item>,
    pub theme: Theme,
    pub goal: Goal,
    /// The exit hatch (adventure), and whether the goal has opened it.
    pub exit: Option<(usize, usize)>,
    pub exit_open: bool,
    pub cleared: bool,
    /// Adventure rules: knocked out costs a life; you lose your special items.
    pub adventure: bool,
    pub species: Vec<crate::baddie::Species>,
    pub baddies: Vec<crate::baddie::Baddie>,
    pub eggs: Vec<Egg>,
    pub sparks: Vec<Spark>,
    pub items: Vec<Vec<Option<Item>>>,
    pub pickups: Vec<Vec<Option<Pickup>>>,
    pub shoves: Vec<Shove>,
    pub flames: Vec<Vec<Option<Flame>>>,
    pub bombs: Vec<Bomb>,
    pub surges: Vec<Surge>,
    /// Percent of bombs that fizzle into UXOs instead of bursting.
    pub fizzle: u32,
    pub players: [Player; MAX_PLAYERS],
    pub steps: u64,
    pub events: Vec<Event>,
    /// Battle rules: knocked-out players stay out (and may ride the border).
    pub battle: bool,
    pub revenge: bool,
    /// A revenge rider whose bomb knocks someone out takes their place.
    pub comeback: bool,
    /// Steps since the round started.
    pub clock: u32,
    fill: Vec<(usize, usize)>,
    fill_next: usize,
    next_id: u32,
    rng: u64,
}

/// Start corners for an arena, in join order.
fn corners(w: usize, h: usize) -> [(usize, usize); MAX_PLAYERS] {
    [(1, 1), (w - 2, h - 2), (w - 2, 1), (1, h - 2)]
}

pub fn centre(t: usize) -> i32 {
    t as i32 * TILE + TILE / 2
}

/// The border tiles clockwise from the top-left corner: the revenge riders' track.
fn ring(w: usize, h: usize) -> Vec<(usize, usize)> {
    let mut r = Vec::new();
    r.extend((0..w).map(|x| (x, 0)));
    r.extend((1..h).map(|y| (w - 1, y)));
    r.extend((0..w - 1).rev().map(|x| (x, h - 1)));
    r.extend((1..h - 1).rev().map(|y| (0, y)));
    r
}

/// A rider's position on screen, in units, from its place along the ring.
fn rider_pos(p: i32, w: usize, h: usize) -> (i32, i32) {
    let r = ring(w, h);
    let n = r.len() as i32;
    let k = p.div_euclid(TILE).rem_euclid(n);
    let f = p.rem_euclid(TILE);
    let (a, b) = (r[k as usize], r[((k + 1) % n) as usize]);
    let lerp = |u: usize, v: usize| centre(u) + (centre(v) - centre(u)) * f / TILE;
    (lerp(a.0, b.0), lerp(a.1, b.1))
}

/// Sudden death fills the arena ring by ring from the outside in, clockwise from the
/// top-left, skipping the pillars.
fn spiral(w: usize, h: usize) -> Vec<(usize, usize)> {
    let (mut x0, mut y0, mut x1, mut y1) = (1i32, 1i32, w as i32 - 2, h as i32 - 2);
    let mut out = Vec::new();
    while x0 <= x1 && y0 <= y1 {
        for x in x0..=x1 {
            out.push((x, y0));
        }
        for y in y0 + 1..=y1 {
            out.push((x1, y));
        }
        if y1 > y0 {
            for x in (x0..x1).rev() {
                out.push((x, y1));
            }
        }
        if x1 > x0 {
            for y in (y0 + 1..y1).rev() {
                out.push((x0, y));
            }
        }
        (x0, y0, x1, y1) = (x0 + 1, y0 + 1, x1 - 1, y1 - 1);
    }
    out.into_iter().map(|(x, y)| (x as usize, y as usize)).filter(|&(x, y)| !(x % 2 == 0 && y % 2 == 0)).collect()
}

impl Game {
    /// An empty arena of `w` x `h` tiles (floor inside a wall).
    fn blank(w: usize, h: usize, seed: u64) -> Game {
        let mut g = Game {
            w,
            h,
            tiles: vec![vec![Tile::Floor; w]; h],
            ground: vec![vec![Ground::Plain; w]; h],
            spawns: corners(w, h),
            item_chance: ITEM_CHANCE,
            hurry_at: HURRY_AT,
            give: Vec::new(),
            theme: Theme::Brick,
            goal: Goal::None,
            exit: None,
            exit_open: false,
            cleared: false,
            adventure: false,
            species: Vec::new(),
            baddies: Vec::new(),
            eggs: Vec::new(),
            sparks: Vec::new(),
            items: vec![vec![None; w]; h],
            pickups: vec![vec![None; w]; h],
            shoves: Vec::new(),
            flames: vec![vec![None; w]; h],
            bombs: Vec::new(),
            surges: Vec::new(),
            fizzle: 0,
            players: std::array::from_fn(|_| Player::new()),
            steps: 0,
            events: Vec::new(),
            battle: false,
            revenge: true,
            comeback: true,
            clock: 0,
            fill: spiral(w, h),
            fill_next: 0,
            next_id: 1,
            rng: seed.max(1),
        };
        for y in 0..h {
            for x in 0..w {
                if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                    g.tiles[y][x] = Tile::Wall;
                }
            }
        }
        g
    }

    /// The classic arena: pillars on a checker, soft blocks three times in four.
    pub fn new(seed: u64) -> Game {
        let mut g = Game::blank(W, H, seed);
        for y in 0..H {
            for x in 0..W {
                if x % 2 == 0 && y % 2 == 0 {
                    g.tiles[y][x] = Tile::Wall;
                }
            }
        }
        for y in 1..H - 1 {
            for x in 1..W - 1 {
                if g.tiles[y][x] == Tile::Floor && !g.near_spawn(x, y) && g.rand(100) < 75 {
                    g.tiles[y][x] = Tile::Soft;
                }
            }
        }
        g
    }

    /// A round on a designed stage.
    pub fn from_stage(st: &crate::stage::Stage, seed: u64) -> Game {
        use crate::stage::Cell;
        let mut g = Game::blank(st.w, st.h, seed);
        g.species = crate::baddie::roster();
        let mut placed: Vec<(String, usize, usize)> = Vec::new();
        for y in 0..st.h {
            for x in 0..st.w {
                g.ground[y][x] = Ground::Plain;
                g.tiles[y][x] = Tile::Floor;
                match st.cells[y][x] {
                    Cell::Wall => g.tiles[y][x] = Tile::Wall,
                    Cell::Soft => g.tiles[y][x] = Tile::Soft,
                    Cell::MaybeSoft => {
                        if g.rand(100) < 75 {
                            g.tiles[y][x] = Tile::Soft;
                        }
                    }
                    Cell::Floor => {}
                    Cell::Start(i) => g.spawns[i] = (x, y),
                    Cell::Belt(d) => g.ground[y][x] = Ground::Belt(d),
                    Cell::Arrow(d) => g.ground[y][x] = Ground::Arrow(d),
                    Cell::Warp => g.ground[y][x] = Ground::Warp,
                    Cell::Ice => g.ground[y][x] = Ground::Ice,
                    Cell::Exit => g.exit = Some((x, y)),
                    Cell::Dud => g.tiles[y][x] = Tile::Dud,
                    Cell::Water => {
                        g.tiles[y][x] = Tile::Wall;
                        g.ground[y][x] = Ground::Water;
                    }
                    Cell::Tree => {
                        g.tiles[y][x] = Tile::Wall;
                        g.ground[y][x] = Ground::Tree;
                    }
                    Cell::Baddie(ref id) => placed.push((id.clone(), x, y)),
                    Cell::Block => g.tiles[y][x] = Tile::Block,
                    Cell::Key(c) => g.pickups[y][x] = Some(Pickup::Key(c)),
                    Cell::Door(c) => g.tiles[y][x] = Tile::Door(c),
                    Cell::Switch => g.ground[y][x] = Ground::Switch,
                    Cell::Gate => {
                        g.tiles[y][x] = Tile::Gate;
                        g.ground[y][x] = Ground::Gate;
                    }
                    Cell::Part => g.pickups[y][x] = Some(Pickup::Part),
                    Cell::Socket => g.tiles[y][x] = Tile::Socket,
                }
            }
        }
        g.item_chance = st.items;
        g.give = st.give.clone();
        g.theme = st.theme;
        g.goal = st.goal;
        for (id, x, y) in placed {
            let si = g.species.iter().position(|s| s.id == id).unwrap_or_else(|| panic!("{}: no baddie {id:?}", st.name));
            let sp = g.species[si].clone();
            let t = g.rand(sp.every.max(60));
            g.baddies.push(crate::baddie::Baddie::new(si, &sp, x, y, t));
        }
        g
    }

    pub(crate) fn rand_pub(&mut self, n: u32) -> u32 {
        self.rand(n)
    }

    fn rand(&mut self, n: u32) -> u32 {
        // xorshift64
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng % n as u64) as u32
    }

    fn roll_item(&mut self) -> Item {
        let total: u32 = DROPS.iter().map(|d| d.1).sum();
        let mut r = self.rand(total);
        for &(item, w) in &DROPS {
            if r < w {
                return item;
            }
            r -= w;
        }
        Item::BombUp
    }

    pub fn join(&mut self, i: usize) {
        let at = self.spawns[i];
        let p = &mut self.players[i];
        *p = Player::new();
        p.joined = true;
        Self::spawn(p, at);
        for item in self.give.clone() {
            self.apply(i, item);
        }
        let p = &mut self.players[i];
        p.picked.clear();
        p.items_got = 0;
    }

    fn spawn(p: &mut Player, (sx, sy): (usize, usize)) {
        p.x = centre(sx);
        p.y = centre(sy);
        p.alive = true;
        p.facing = Dir::Down;
    }

    /// Sudden death has begun.
    pub fn hurry(&self) -> bool {
        self.battle && self.clock >= self.hurry_at
    }

    /// The grounded bomb on a tile, if any.
    pub fn bomb_at(&self, x: usize, y: usize) -> Option<&Bomb> {
        self.bombs.iter().find(|b| b.grounded() && b.tile() == (x, y))
    }

    fn bomb_index_at(&self, x: usize, y: usize) -> Option<usize> {
        self.bombs.iter().position(|b| b.grounded() && b.tile() == (x, y))
    }

    /// Can player `i` walk into tile (x, y)?
    fn passable(&self, i: usize, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return false;
        }
        let (x, y) = (x as usize, y as usize);
        let p = &self.players[i];
        let ground = match self.tiles[y][x] {
            Tile::Floor => true,
            Tile::Soft => p.block_pass,
            _ => false,
        };
        ground && (p.bomb_pass || self.bomb_at(x, y).is_none_or(|b| b.pass[i]))
    }

    /// Does player `i`'s body (one tile in size) overlap tile (x, y)?
    fn overlaps(&self, i: usize, x: usize, y: usize) -> bool {
        let p = &self.players[i];
        p.alive && (p.x - centre(x)).abs() < TILE && (p.y - centre(y)).abs() < TILE
    }

    fn passes(&self, x: usize, y: usize) -> [bool; MAX_PLAYERS] {
        std::array::from_fn(|j| self.overlaps(j, x, y))
    }

    fn new_bomb(&mut self, owner: usize, x: usize, y: usize, fire: u32, kind: BombKind, fuse: u32) -> Bomb {
        let id = self.next_id;
        self.next_id += 1;
        Bomb {
            id,
            x: centre(x),
            y: centre(y),
            fuse,
            owner,
            fire,
            kind,
            motion: Motion::Still,
            armed: false,
            age: 0,
            from_rider: false,
            pass: self.passes(x, y),
            uxo: None,
            jolted: false,
            dud: false,
            loose: false,
        }
    }

    /// Player `i` drops a bomb on tile (x, y), with their fire, kind and disease.
    pub fn place_bomb(&mut self, i: usize, x: usize, y: usize) -> bool {
        if self.tiles[y][x] != Tile::Floor || self.bomb_at(x, y).is_some() {
            return false;
        }
        let p = &self.players[i];
        let fire = if p.disease == Some(Disease::MinFire) { 1 } else { p.fire };
        let fuse = match p.disease {
            Some(Disease::ShortFuse) => FUSE * 2 / 5,
            Some(Disease::LongFuse) => FUSE * 2,
            _ => FUSE,
        };
        let dud = p.disease == Some(Disease::Duds);
        let mut b = self.new_bomb(i, x, y, fire, p.kind, fuse);
        b.dud = dud;
        self.bombs.push(b);
        self.players[i].dropped += 1;
        true
    }

    fn owned(&self, i: usize) -> u32 {
        self.bombs.iter().filter(|b| b.owner == i && !b.from_rider && !b.loose).count() as u32
    }

    /// Advance one step with each player's command.
    pub fn step(&mut self, cmds: &[Cmd; MAX_PLAYERS]) {
        self.steps += 1;
        self.clock += 1;
        self.sudden_death();
        for i in 0..MAX_PLAYERS {
            if !self.players[i].joined {
                continue;
            }
            if self.players[i].rider.is_some() {
                self.ride(i, cmds[i]);
                continue;
            }
            self.tick_player(i);
            let p = &self.players[i];
            if !p.alive {
                continue;
            }
            if p.stun == 0 {
                self.control(i, cmds[i]);
            }
            self.ground_effects(i);
        }
        // stepping off a bomb closes it behind you
        for b in 0..self.bombs.len() {
            let (bx, by) = self.bombs[b].tile();
            for j in 0..MAX_PLAYERS {
                if self.bombs[b].pass[j] && !self.overlaps(j, bx, by) {
                    self.bombs[b].pass[j] = false;
                }
            }
        }
        self.tick_puzzles();
        self.move_bombs();
        self.tick_bombs();
        self.tick_surges();
        self.tick_board();
        self.step_baddies();
        self.hurt_and_pickup();
        self.infect();
        self.take_eggs();
        self.check_goal();
        self.step_sparks();
    }

    fn take_eggs(&mut self) {
        for i in 0..MAX_PLAYERS {
            let p = &self.players[i];
            if !p.joined || !p.alive {
                continue;
            }
            let (px, py) = (p.x, p.y);
            if let Some(k) = self.eggs.iter().position(|e| (e.x - px).abs() < TILE * 2 / 3 && (e.y - py).abs() < TILE * 2 / 3) {
                let e = self.eggs.swap_remove(k);
                let p = &mut self.players[i];
                if p.mount.is_none() {
                    p.mount = Some(e.species);
                    self.events.push(Event::Mounted { player: i });
                } else if p.spare.len() < 2 {
                    p.spare.push(e.species);
                }
            }
        }
    }

    fn check_goal(&mut self) {
        if self.cleared {
            return;
        }
        let met = match self.goal {
            Goal::None => false,
            Goal::Clear => self.baddies.iter().all(|b| !b.alive),
            Goal::Duds => !self.tiles.iter().flatten().any(|&t| t == Tile::Dud),
            Goal::Boss => {
                let boss = self.species.iter().position(|s| s.size > 1);
                !self.baddies.iter().any(|b| b.alive && Some(b.species) == boss)
            }
        };
        if met && self.goal == Goal::Boss {
            self.cleared = true;
            self.events.push(Event::Cleared);
            return;
        }
        if met && !self.exit_open && self.exit.is_some() {
            self.exit_open = true;
            self.events.push(Event::ExitOpen);
        }
        if self.exit_open
            && let Some((ex, ey)) = self.exit
            && self.players.iter().any(|p| p.joined && p.alive && p.tile() == (ex, ey) && (p.x - centre(ex)).abs() < TILE / 3 && (p.y - centre(ey)).abs() < TILE / 3)
        {
            self.cleared = true;
            self.events.push(Event::Cleared);
        }
    }

    /// A burst of firework sparks at a world point (units).
    pub fn fireworks(&mut self, x: i32, y: i32, n: u32) {
        let hue = self.rand(6) as u8;
        for k in 0..n {
            let a = k as f32 / n as f32 * std::f32::consts::TAU + self.rand(100) as f32 / 300.0;
            let v = 40.0 + self.rand(40) as f32;
            let life = 40 + self.rand(25);
            self.sparks.push(Spark { x, y, vx: (a.cos() * v) as i32, vy: (a.sin() * v) as i32 - 20, life, hue });
        }
    }

    fn step_sparks(&mut self) {
        for s in &mut self.sparks {
            s.x += s.vx / 2;
            s.y += s.vy / 2;
            s.vy += 3;
            s.vx = s.vx * 95 / 100;
            s.life -= 1;
        }
        self.sparks.retain(|s| s.life > 0);
    }

    fn tick_player(&mut self, i: usize) {
        let at = self.spawns[i];
        let p = &mut self.players[i];
        if !p.alive {
            if !p.out {
                p.respawn = p.respawn.saturating_sub(1);
                if p.respawn == 0 {
                    Self::spawn(p, at);
                    p.invuln = INVULN;
                }
            }
            return;
        }
        p.invuln = p.invuln.saturating_sub(1);
        p.vest = p.vest.saturating_sub(1);
        p.stun = p.stun.saturating_sub(1);
        p.infect_cool = p.infect_cool.saturating_sub(1);
        if p.disease.is_some() {
            p.disease_left = p.disease_left.saturating_sub(1);
            if p.disease_left == 0 {
                p.disease = None;
            }
        }
    }

    fn control(&mut self, i: usize, c: Cmd) {
        let mut c = c;
        // mid-hop on a mount: carried through the air, nothing else
        if let Some((from, to, t, dur)) = self.players[i].hop {
            let t = t + 1;
            let p = &mut self.players[i];
            if t >= dur {
                (p.x, p.y) = to;
                p.hop = None;
            } else {
                p.x = from.0 + (to.0 - from.0) * t as i32 / dur as i32;
                p.y = from.1 + (to.1 - from.1) * t as i32 / dur as i32;
                p.hop = Some((from, to, t, dur));
            }
            return;
        }
        if c.ride {
            self.ride_ability(i);
        }
        if self.players[i].dash > 0 {
            let d = self.players[i].facing;
            self.players[i].dash -= 1;
            let moved = self.walk(i, d, BASE_SPEED * 3);
            if !moved {
                self.players[i].dash = 0;
            }
            return;
        }
        let p = &self.players[i];
        if p.disease == Some(Disease::Reverse) {
            c.primary = c.primary.map(Dir::back);
            c.secondary = c.secondary.map(Dir::back);
        }
        if p.disease == Some(Disease::NoStop) && c.primary.is_none() {
            c.primary = Some(p.facing);
        }
        // on ice you keep sliding the way you were going
        if let Some(d) = p.ice {
            c.primary = Some(d);
            c.secondary = None;
        }
        let mut moved = false;
        if let Some(d) = c.primary {
            self.players[i].facing = d;
            let speed = self.players[i].walk_speed();
            moved = self.walk(i, d, speed);
            if !moved {
                self.try_kick(i, d);
                self.try_push(i, d);
            } else {
                self.players[i].push = 0;
            }
            if !moved && let Some(s) = c.secondary {
                moved = self.walk(i, s, speed);
                if moved {
                    self.players[i].facing = s;
                }
            }
        }
        if c.primary.is_none() {
            self.players[i].push = 0;
        }
        let (tx, ty) = self.players[i].tile();
        let on_ice = self.ground[ty][tx] == Ground::Ice;
        let p = &mut self.players[i];
        p.ice = if on_ice && moved { Some(p.facing) } else { None };
        p.walk = if moved { p.walk + 1 } else { 0 };
        let auto = p.disease == Some(Disease::AutoDrop);
        if c.bomb || auto {
            self.drop_bomb(i, c.bomb);
        }
        if c.action {
            self.action(i);
        }
    }

    fn drop_bomb(&mut self, i: usize, pressed: bool) {
        let p = &self.players[i];
        if p.disease == Some(Disease::NoBombs) || self.bombs.iter().any(|b| b.motion == Motion::Carried { by: i }) {
            return;
        }
        let (tx, ty) = p.tile();
        let (line, facing) = (p.line, p.facing);
        // line bomb: pressing again on your own bomb lays the rest in a row ahead
        if pressed && line && self.bomb_at(tx, ty).is_some_and(|b| b.owner == i) {
            let (dx, dy) = facing.vec();
            let (mut x, mut y) = (tx as i32 + dx, ty as i32 + dy);
            let mut laid = 0;
            while self.owned(i) < self.players[i].bombs && x > 0 && y > 0 && x < self.w as i32 - 1 && y < self.h as i32 - 1 {
                if self.players.iter().any(|q| q.alive && q.tile() == (x as usize, y as usize)) || !self.place_bomb(i, x as usize, y as usize) {
                    break;
                }
                laid += 1;
                (x, y) = (x + dx, y + dy);
            }
            if laid > 0 {
                self.events.push(Event::Drop { player: i });
            }
            return;
        }
        if self.owned(i) < self.players[i].bombs && self.place_bomb(i, tx, ty) {
            self.events.push(Event::Drop { player: i });
        }
    }

    /// The mount button: hop two tiles over anything (onto free floor), or dash.
    fn ride_ability(&mut self, i: usize) {
        let Some(m) = self.players[i].mount else { return };
        let ride = self.species[m].egg;
        let p = &self.players[i];
        let (tx, ty) = p.tile();
        let (dx, dy) = p.facing.vec();
        match ride {
            Some(crate::baddie::Ride::Hop) => {
                let (lx, ly) = (tx as i32 + dx * 2, ty as i32 + dy * 2);
                if self.passable(i, lx, ly) && lx > 0 && ly > 0 {
                    let from = (p.x, p.y);
                    self.players[i].hop = Some((from, (centre(lx as usize), centre(ly as usize)), 0, 24));
                }
            }
            Some(crate::baddie::Ride::Dash) => self.players[i].dash = 40,
            None => {}
        }
    }

    /// Belts carry you; warps send you on.
    fn ground_effects(&mut self, i: usize) {
        let (tx, ty) = self.players[i].tile();
        match self.ground[ty][tx] {
            Ground::Belt(d) => {
                self.walk(i, d, BELT_SPEED);
            }
            Ground::Warp => {
                let p = &self.players[i];
                let near = (p.x - centre(tx)).abs() < TILE / 4 && (p.y - centre(ty)).abs() < TILE / 4;
                if !p.warped && near {
                    // the next warp in reading order that is still open
                    let w = self.w;
                    let warps: Vec<(usize, usize)> = (0..self.h)
                        .flat_map(|y| (0..w).map(move |x| (x, y)))
                        .filter(|&(x, y)| self.ground[y][x] == Ground::Warp && self.tiles[y][x] == Tile::Floor)
                        .collect();
                    if let Some(k) = warps.iter().position(|&w| w == (tx, ty))
                        && warps.len() > 1
                    {
                        let (nx, ny) = warps[(k + 1) % warps.len()];
                        let p = &mut self.players[i];
                        (p.x, p.y) = (centre(nx), centre(ny));
                        p.warped = true;
                        p.ice = None;
                        self.events.push(Event::Warp { player: i });
                    }
                }
                return;
            }
            _ => {}
        }
        self.players[i].warped = false;
    }

    /// The action button: throw what you carry, punch the bomb ahead, pick up the one
    /// underfoot, or set off your oldest remote bomb, in that order.
    fn action(&mut self, i: usize) {
        let p = &self.players[i];
        let (tx, ty) = p.tile();
        let (dx, dy) = p.facing.vec();
        let (px, py, punch, glove) = (p.x, p.y, p.punch, p.glove);
        if let Some(k) = self.bombs.iter().position(|b| b.motion == Motion::Carried { by: i }) {
            let to = (tx as i32 + dx * THROW_TILES, ty as i32 + dy * THROW_TILES);
            self.launch(k, (px, py), to, dx, dy, THROW_TILES as u32 * FLY_STEPS_PER_TILE);
            self.events.push(Event::Throw { player: i });
            return;
        }
        let ahead = (tx as i32 + dx, ty as i32 + dy);
        if punch && let Some(k) = self.bomb_index_at(ahead.0 as usize, ahead.1 as usize) {
            let to = (ahead.0 + dx * THROW_TILES, ahead.1 + dy * THROW_TILES);
            let from = (self.bombs[k].x, self.bombs[k].y);
            self.launch(k, from, to, dx, dy, THROW_TILES as u32 * FLY_STEPS_PER_TILE);
            self.events.push(Event::Throw { player: i });
            return;
        }
        if glove && let Some(k) = self.bomb_index_at(tx, ty) {
            self.bombs[k].motion = Motion::Carried { by: i };
            return;
        }
        if let Some(k) = self
            .bombs
            .iter()
            .enumerate()
            .filter(|(_, b)| b.owner == i && b.kind == BombKind::Remote && b.grounded())
            .min_by_key(|(_, b)| b.id)
            .map(|(k, _)| k)
        {
            self.bombs[k].fuse = 1;
        }
    }

    /// Send bomb `k` flying from `from` (units) to tile `to`, wrapping round the arena.
    fn launch(&mut self, k: usize, from: (i32, i32), to: (i32, i32), dx: i32, dy: i32, dur: u32) {
        let to = self.wrap(to);
        self.bombs[k].motion = Motion::Fly { from, to: (centre(to.0 as usize), centre(to.1 as usize)), t: 0, dur, dx, dy };
    }

    /// Walking into a still bomb with the kick sends it sliding.
    fn try_kick(&mut self, i: usize, d: Dir) {
        let p = &self.players[i];
        if !p.kick || p.bomb_pass {
            return;
        }
        let (dx, dy) = d.vec();
        let (tx, ty) = p.tile();
        // touching the bomb: at or past the middle of your own tile, lined up with it
        let (along, mid, off) = if dx != 0 { (p.x, centre(tx), p.y - centre(ty)) } else { (p.y, centre(ty), p.x - centre(tx)) };
        if (along - mid) * (dx + dy) < 0 || off.abs() > TILE / 4 {
            return;
        }
        let (ax, ay) = (tx as i32 + dx, ty as i32 + dy);
        if let Some(k) = self.bomb_index_at(ax as usize, ay as usize)
            && self.bombs[k].motion == Motion::Still
        {
            self.bombs[k].motion = Motion::Slide { dx, dy, speed: KICK_SPEED };
            self.events.push(Event::Kick { player: i });
        }
    }

    /// Walking into a door with its key opens it; leaning on a block a moment pushes it.
    fn try_push(&mut self, i: usize, d: Dir) {
        let p = &self.players[i];
        let (dx, dy) = d.vec();
        let (tx, ty) = p.tile();
        // touching it: at or past the middle of your own tile, lined up with it
        let (along, mid, off) = if dx != 0 { (p.x, centre(tx), p.y - centre(ty)) } else { (p.y, centre(ty), p.x - centre(tx)) };
        if (along - mid) * (dx + dy) < 0 || off.abs() > TILE / 4 {
            self.players[i].push = 0;
            return;
        }
        let (ax, ay) = ((tx as i32 + dx) as usize, (ty as i32 + dy) as usize);
        match self.tiles[ay][ax] {
            Tile::Door(c) if p.keys[c as usize] > 0 => {
                self.players[i].keys[c as usize] -= 1;
                self.tiles[ay][ax] = Tile::Floor;
                self.events.push(Event::DoorOpened { player: i });
            }
            Tile::Block if !self.shoves.iter().any(|s| s.to == (ax, ay)) => {
                self.players[i].push += 1;
                if self.players[i].push >= PUSH_DELAY && self.shove(ax, ay, d) {
                    self.players[i].push = 0;
                    self.events.push(Event::Shoved { player: i });
                }
            }
            _ => self.players[i].push = 0,
        }
    }

    /// Can a block move into tile (x, y)? Onto empty floor, or into water to sink.
    fn block_free(&self, x: usize, y: usize) -> bool {
        let water = self.tiles[y][x] == Tile::Wall && self.ground[y][x] == Ground::Water;
        (self.tiles[y][x] == Tile::Floor || water)
            && self.ground[y][x] != Ground::Warp
            && self.exit != Some((x, y))
            && self.pickups[y][x].is_none()
            && !self.occupied(x, y)
    }

    /// Is anyone or anything standing on tile (x, y): a player, a baddie, a bomb?
    fn occupied(&self, x: usize, y: usize) -> bool {
        (0..MAX_PLAYERS).any(|j| self.players[j].joined && self.overlaps(j, x, y))
            || self.bomb_at(x, y).is_some()
            || self.baddies.iter().any(|b| {
                let (bx, by) = b.tile();
                let n = self.species[b.species].size as usize;
                b.alive && (bx..bx + n).contains(&x) && (by..by + n).contains(&y)
            })
    }

    /// Start the block on (x, y) moving one tile toward `d`, if it can.
    fn shove(&mut self, x: usize, y: usize, d: Dir) -> bool {
        let (dx, dy) = d.vec();
        let (nx, ny) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
        if !self.block_free(nx, ny) {
            return false;
        }
        self.tiles[y][x] = Tile::Floor;
        self.tiles[ny][nx] = Tile::Block;
        self.items[ny][nx] = None;
        self.shoves.push(Shove { from: (x, y), to: (nx, ny), dir: d, t: 0 });
        true
    }

    /// Blocks in motion arrive; in water they sink, on ice they slide on, belts carry
    /// them. Then gates follow the switches and the socket the parts.
    fn tick_puzzles(&mut self) {
        let mut arrived = Vec::new();
        for s in &mut self.shoves {
            s.t += 1;
            if s.t >= SHOVE_STEPS {
                arrived.push(*s);
            }
        }
        self.shoves.retain(|s| s.t < SHOVE_STEPS);
        for s in arrived {
            let (x, y) = s.to;
            match self.ground[y][x] {
                Ground::Water => {
                    self.tiles[y][x] = Tile::Floor;
                    self.ground[y][x] = Ground::Sunk;
                }
                Ground::Ice => {
                    self.shove(x, y, s.dir);
                }
                _ => {}
            }
        }
        for y in 0..self.h {
            for x in 0..self.w {
                if let Ground::Belt(d) = self.ground[y][x]
                    && self.tiles[y][x] == Tile::Block
                    && !self.shoves.iter().any(|s| s.to == (x, y))
                {
                    self.shove(x, y, d);
                }
            }
        }
        let switches: Vec<(usize, usize)> =
            (0..self.h).flat_map(|y| (0..self.w).map(move |x| (x, y))).filter(|&(x, y)| self.ground[y][x] == Ground::Switch).collect();
        if !switches.is_empty() {
            let open = switches.iter().all(|&(x, y)| self.tiles[y][x] == Tile::Block || self.occupied(x, y));
            let mut changed = false;
            for y in 0..self.h {
                for x in 0..self.w {
                    if self.ground[y][x] != Ground::Gate {
                        continue;
                    }
                    if open && self.tiles[y][x] == Tile::Gate {
                        self.tiles[y][x] = Tile::Floor;
                        changed = true;
                    } else if !open && self.tiles[y][x] == Tile::Floor && !self.occupied(x, y) && self.flames[y][x].is_none() {
                        // a gate won't shut on anyone standing in it
                        self.tiles[y][x] = Tile::Gate;
                        changed = true;
                    }
                }
            }
            if changed {
                self.events.push(Event::Gates { open });
            }
        }
        if self.tiles.iter().flatten().any(|&t| t == Tile::Socket) && !self.pickups.iter().flatten().any(|&p| p == Some(Pickup::Part)) {
            for t in self.tiles.iter_mut().flatten() {
                if *t == Tile::Socket {
                    *t = Tile::Floor;
                }
            }
            self.events.push(Event::SocketOpen);
        }
    }

    /// Move player `i` up to `amount` units toward `d`, with lane alignment and corner
    /// assist. Returns whether they moved at all.
    fn walk(&mut self, i: usize, d: Dir, amount: i32) -> bool {
        let (dx, dy) = d.vec();
        let horizontal = dx != 0;
        let sgn = dx + dy;
        let p = &self.players[i];
        let (along, perp) = if horizontal { (p.x, p.y) } else { (p.y, p.x) };
        let t_along = along.div_euclid(TILE);
        let lane = perp.div_euclid(TILE);
        let lane_c = lane * TILE + TILE / 2;
        // tile (along, lane) to board (x, y)
        let at = |a: i32, l: i32| if horizontal { (a, l) } else { (l, a) };
        let free = |g: &Game, a: i32, l: i32| {
            let (x, y) = at(a, l);
            g.passable(i, x, y)
        };
        let (mut new_along, mut new_perp) = (along, perp);
        let mut left = amount;
        let ahead_free = free(self, t_along + sgn, lane);

        // up to the middle of the current tile you can always go
        let to_mid = (t_along * TILE + TILE / 2 - along) * sgn;
        if to_mid > 0 {
            let s = left.min(to_mid);
            new_along += s * sgn;
            if ahead_free {
                let off = new_perp - lane_c;
                new_perp -= off.signum() * s.min(off.abs());
            }
            left -= s;
        }
        if left > 0 {
            let off = new_perp - lane_c;
            if ahead_free {
                // line up with the lane, then carry on
                let al = left.min(off.abs());
                new_perp -= off.signum() * al;
                left -= al;
                new_along += left * sgn;
            } else if off.abs() >= ASSIST_MIN {
                // corner assist: slide toward the neighbouring lane if it is open ahead
                let side = off.signum();
                if free(self, t_along + sgn, lane + side) && free(self, t_along, lane + side) {
                    new_perp += side * left.min(TILE - off.abs());
                }
            } else if off != 0 {
                new_perp -= off.signum() * left.min(off.abs());
            }
        }
        let moved = (new_along, new_perp) != (along, perp);
        let p = &mut self.players[i];
        if horizontal {
            (p.x, p.y) = (new_along, new_perp);
        } else {
            (p.y, p.x) = (new_along, new_perp);
        }
        moved
    }

    /// Is tile (x, y) blocked for a bomb landing or sliding into it?
    fn bomb_blocked(&self, x: i32, y: i32, skip: u32) -> bool {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return true;
        }
        let (x, y) = (x as usize, y as usize);
        self.tiles[y][x] != Tile::Floor || self.bombs.iter().any(|b| b.id != skip && b.grounded() && b.tile() == (x, y))
    }

    fn player_on(&self, x: usize, y: usize) -> Option<usize> {
        (0..MAX_PLAYERS).find(|&j| self.players[j].alive && self.players[j].tile() == (x, y))
    }

    fn move_bombs(&mut self) {
        for k in 0..self.bombs.len() {
            let id = self.bombs[k].id;
            match self.bombs[k].motion {
                Motion::Still => {
                    // a belt picks up a bomb resting on it
                    let (tx, ty) = self.bombs[k].tile();
                    if let Ground::Belt(d) = self.ground[ty][tx] {
                        let (dx, dy) = d.vec();
                        self.bombs[k].motion = Motion::Slide { dx, dy, speed: BELT_SPEED };
                    }
                }
                Motion::Carried { by } => {
                    let p = &self.players[by];
                    let (x, y) = (p.x, p.y);
                    let b = &mut self.bombs[k];
                    (b.x, b.y) = (x, y);
                }
                Motion::Slide { dx, dy, speed } => {
                    let b = &self.bombs[k];
                    let (tx, ty) = b.tile();
                    let (along, mid) = if dx != 0 { (b.x, centre(tx)) } else { (b.y, centre(ty)) };
                    let sgn = dx + dy;
                    let before = (mid - along) * sgn;
                    // reaching the middle of a tile: turn at arrows (and belts), stop where
                    // a belt ends, and look ahead before going on
                    let (mut dx, mut dy) = (dx, dy);
                    if before >= 0 && before < speed {
                        let turn = match self.ground[ty][tx] {
                            Ground::Arrow(d) => Some(d.vec()),
                            Ground::Belt(d) if speed == BELT_SPEED => Some(d.vec()),
                            _ if speed == BELT_SPEED => {
                                let pass = self.passes(tx, ty);
                                let b = &mut self.bombs[k];
                                (b.x, b.y) = (centre(tx), centre(ty));
                                b.motion = Motion::Still;
                                b.pass = pass;
                                continue;
                            }
                            _ => None,
                        };
                        if let Some(v) = turn
                            && v != (dx, dy)
                        {
                            (dx, dy) = v;
                            let b = &mut self.bombs[k];
                            (b.x, b.y) = (centre(tx), centre(ty));
                            b.motion = Motion::Slide { dx, dy, speed };
                        }
                        let b = &self.bombs[k];
                        let (ax, ay) = (tx as i32 + dx, ty as i32 + dy);
                        let person = ax >= 0 && ay >= 0 && self.player_on(ax as usize, ay as usize).is_some();
                        if self.bomb_blocked(ax, ay, id) || person {
                            let rubber = b.kind == BombKind::Rubber;
                            let (bx, by) = (tx as i32 - dx, ty as i32 - dy);
                            let b = &mut self.bombs[k];
                            (b.x, b.y) = (centre(tx), centre(ty));
                            if rubber && speed != BELT_SPEED && !self.bomb_blocked(bx, by, id) {
                                self.bombs[k].motion = Motion::Slide { dx: -dx, dy: -dy, speed };
                            } else {
                                let pass = self.passes(tx, ty);
                                self.bombs[k].motion = Motion::Still;
                                self.bombs[k].pass = pass;
                            }
                            continue;
                        }
                    }
                    let b = &mut self.bombs[k];
                    b.x += dx * speed;
                    b.y += dy * speed;
                }
                Motion::Fly { from, to, t, dur, dx, dy } => {
                    let t = t + 1;
                    if t < dur {
                        let b = &mut self.bombs[k];
                        b.x = from.0 + (to.0 - from.0) * t as i32 / dur as i32;
                        b.y = from.1 + (to.1 - from.1) * t as i32 / dur as i32;
                        b.motion = Motion::Fly { from, to, t, dur, dx, dy };
                        continue;
                    }
                    let (tx, ty) = (to.0.div_euclid(TILE), to.1.div_euclid(TILE));
                    let hit = self.player_on(tx as usize, ty as usize);
                    if let Some(j) = hit {
                        // a bomb landing on your head knocks you dizzy
                        if self.players[j].stun == 0 {
                            self.events.push(Event::Stunned { player: j });
                        }
                        self.players[j].stun = STUN;
                    }
                    if hit.is_some() || self.bomb_blocked(tx, ty, id) {
                        // bounce on, one tile at a time
                        let next = self.wrap((tx + dx, ty + dy));
                        let b = &mut self.bombs[k];
                        (b.x, b.y) = to;
                        b.motion = Motion::Fly { from: to, to: (centre(next.0 as usize), centre(next.1 as usize)), t: 0, dur: BOUNCE_STEPS, dx, dy };
                    } else {
                        let pass = self.passes(tx as usize, ty as usize);
                        let b = &mut self.bombs[k];
                        (b.x, b.y) = to;
                        b.motion = Motion::Still;
                        b.pass = pass;
                    }
                }
            }
        }
    }

    fn tick_bombs(&mut self) {
        let mut burst: Vec<u32> = Vec::new();
        for k in 0..self.bombs.len() {
            if self.bombs[k].uxo.is_some() && self.bombs[k].motion != Motion::Still {
                self.bombs[k].jolted = true;
            }
            let b = &self.bombs[k];
            if !b.grounded() {
                continue;
            }
            let (tx, ty) = b.tile();
            let burning = self.flames[ty][tx].is_some();
            // a UXO goes off when a blast reaches it, once it is at rest after being
            // moved, or by itself when its time comes
            if let Some(left) = b.uxo {
                let rest = b.jolted && b.motion == Motion::Still;
                if left <= 1 || rest || burning {
                    burst.push(b.id);
                }
                self.bombs[k].uxo = Some(left.saturating_sub(1));
                continue;
            }
            let owner_gone = !self.players[b.owner].alive && !b.from_rider;
            let b = &mut self.bombs[k];
            b.age += 1;
            match b.kind {
                // a remote bomb waits for its owner, unless they are gone
                BombKind::Remote if b.fuse != 1 && !owner_gone => {}
                BombKind::Mine if b.fuse > MINE_TRIGGER => {
                    if !b.armed && b.age >= MINE_ARM {
                        b.armed = true;
                    }
                }
                _ => b.fuse = b.fuse.saturating_sub(1),
            }
            if b.fuse == 0 && !burning && self.fizzles(k) {
                continue;
            }
            let b = &self.bombs[k];
            if b.fuse == 0 || burning {
                burst.push(b.id);
            }
        }
        // mines go off when someone steps onto them
        for k in 0..self.bombs.len() {
            let b = &self.bombs[k];
            if b.kind == BombKind::Mine && b.armed && b.fuse > MINE_TRIGGER {
                let (tx, ty) = b.tile();
                if (0..MAX_PLAYERS).any(|j| self.overlaps(j, tx, ty) && !b.pass[j] && self.players[j].tile() == (tx, ty)) {
                    self.bombs[k].fuse = MINE_TRIGGER;
                    self.bombs[k].armed = false;
                }
            }
        }
        // chain reactions: each burst can set off more bombs this same step
        while let Some(id) = burst.pop() {
            let Some(k) = self.bombs.iter().position(|b| b.id == id) else { continue };
            let b = self.bombs.swap_remove(k);
            let (x, y) = b.tile();
            self.events.push(Event::Boom { x, y });
            if b.kind == BombKind::Surge {
                self.flame(x, y, 0, 0, b.owner, true);
                let reach = surge_reach(b.fire);
                self.surges.push(Surge { owner: b.owner, left: reach, wait: 0, front: vec![(x, y)], seen: vec![(x, y)] });
                continue;
            }
            self.flame(x, y, UP | DOWN | LEFT | RIGHT, 0, b.owner, false);
            let pierce = b.kind == BombKind::Pierce;
            for d in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
                let (dx, dy) = d.vec();
                for r in 1..=b.fire as i32 {
                    let (tx, ty) = ((x as i32 + dx * r) as usize, (y as i32 + dy * r) as usize);
                    match self.tiles[ty][tx] {
                        Tile::Wall | Tile::Pressure(_) | Tile::Block | Tile::Door(_) | Tile::Gate | Tile::Socket => break,
                        Tile::Dud => {
                            let id = self.set_off_charge(tx, ty, b.owner);
                            burst.push(id);
                            break;
                        }
                        Tile::Soft => {
                            self.tiles[ty][tx] = Tile::Burning(BURN);
                            self.players[b.owner].blocks += 1;
                            if pierce {
                                continue;
                            }
                            break;
                        }
                        Tile::Burning(_) => {
                            if pierce {
                                continue;
                            }
                            break;
                        }
                        Tile::Floor => {}
                    }
                    let end = r == b.fire as i32;
                    self.flame(tx, ty, d.back().bit(), if end { 0 } else { d.bit() }, b.owner, false);
                    if let Some(o) = self.bombs.iter().find(|o| o.grounded() && o.tile() == (tx, ty)) {
                        burst.push(o.id);
                        break;
                    }
                    if self.items[ty][tx].take().is_some() && !pierce {
                        break;
                    }
                }
            }
        }
    }

    /// Does bomb `k`, its fuse run down, fizzle into a UXO instead of bursting?
    fn fizzles(&mut self, k: usize) -> bool {
        let b = &self.bombs[k];
        if b.loose || matches!(b.kind, BombKind::Remote | BombKind::Mine) {
            return false;
        }
        if !b.dud && (self.fizzle == 0 || self.rand(100) >= self.fizzle) {
            return false;
        }
        let left = UXO_MIN + self.rand(UXO_SPAN);
        let b = &mut self.bombs[k];
        b.uxo = Some(left);
        b.loose = true;
        let (x, y) = b.tile();
        self.events.push(Event::Fizzled { x, y });
        true
    }

    /// A blast reached one of purged's charges: it goes off with a real blast of its
    /// own, a step later for a surge, at once in a cross's chain. Returns its bomb.
    fn set_off_charge(&mut self, x: usize, y: usize, by: usize) -> u32 {
        self.tiles[y][x] = Tile::Floor;
        self.events.push(Event::DudPopped { x, y, by });
        self.fireworks(centre(x), centre(y), 28);
        let mut c = self.new_bomb(by, x, y, CHARGE_FIRE, BombKind::Normal, 1);
        c.loose = true;
        let id = c.id;
        self.bombs.push(c);
        id
    }

    fn flame(&mut self, x: usize, y: usize, a: u8, b: u8, owner: usize, surge: bool) {
        let f = self.flames[y][x].get_or_insert(Flame { left: FLAME, links: 0, owner, surge });
        f.left = FLAME;
        f.links |= a | b;
        f.owner = owner;
        f.surge = surge;
    }

    /// Spread each surge one ring further. Walls stop it; it breaks a soft block, pops a
    /// dud, burns an item or reaches a bomb (which then goes off) and goes no further
    /// that way, but flows round everything else.
    fn tick_surges(&mut self) {
        let mut surges = std::mem::take(&mut self.surges);
        for s in &mut surges {
            if s.wait > 0 {
                s.wait -= 1;
                continue;
            }
            s.wait = SURGE_RING - 1;
            let mut next = Vec::new();
            for (x, y) in std::mem::take(&mut s.front) {
                for d in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
                    let (dx, dy) = d.vec();
                    let (tx, ty) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
                    if s.seen.contains(&(tx, ty)) {
                        continue;
                    }
                    s.seen.push((tx, ty));
                    match self.tiles[ty][tx] {
                        Tile::Wall | Tile::Pressure(_) | Tile::Burning(_) | Tile::Block | Tile::Door(_) | Tile::Gate | Tile::Socket => continue,
                        Tile::Dud => {
                            self.set_off_charge(tx, ty, s.owner);
                            continue;
                        }
                        Tile::Soft => {
                            self.tiles[ty][tx] = Tile::Burning(BURN);
                            self.players[s.owner].blocks += 1;
                            continue;
                        }
                        Tile::Floor => {}
                    }
                    // one sheet: link to every neighbour this surge already lit
                    let mut links = 0;
                    for e in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
                        let (ex, ey) = e.vec();
                        let (nx, ny) = ((tx as i32 + ex) as usize, (ty as i32 + ey) as usize);
                        if let Some(f) = &mut self.flames[ny][nx] {
                            if f.surge && s.seen.contains(&(nx, ny)) {
                                f.links |= e.back().bit();
                                links |= e.bit();
                            }
                        }
                    }
                    self.flame(tx, ty, links, 0, s.owner, true);
                    let bomb = self.bombs.iter().any(|o| o.grounded() && o.tile() == (tx, ty));
                    if bomb || self.items[ty][tx].take().is_some() {
                        continue;
                    }
                    next.push((tx, ty));
                }
            }
            s.front = next;
            s.left -= 1;
        }
        surges.retain(|s| s.left > 0 && !s.front.is_empty());
        self.surges = surges;
    }

    fn tick_board(&mut self) {
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(f) = &mut self.flames[y][x] {
                    f.left -= 1;
                    if f.left == 0 {
                        self.flames[y][x] = None;
                    }
                }
                match self.tiles[y][x] {
                    Tile::Burning(t) if t <= 1 => {
                        self.tiles[y][x] = Tile::Floor;
                        if self.rand(100) < self.item_chance {
                            self.items[y][x] = Some(self.roll_item());
                        }
                    }
                    Tile::Burning(t) => self.tiles[y][x] = Tile::Burning(t - 1),
                    Tile::Pressure(t) if t < DROP => self.tiles[y][x] = Tile::Pressure(t + 1),
                    _ => {}
                }
            }
        }
    }

    fn sudden_death(&mut self) {
        if !self.battle || self.clock < self.hurry_at {
            return;
        }
        if self.clock == self.hurry_at {
            self.events.push(Event::Hurry);
            // the riders go home when the walls start closing in
            for p in &mut self.players {
                p.rider = None;
            }
        }
        let every = (FILL_EVERY * 113 / self.fill.len().max(1) as u32).max(4);
        if (self.clock - self.hurry_at) % every != 0 || self.fill_next >= self.fill.len() {
            return;
        }
        let (x, y) = self.fill[self.fill_next];
        self.fill_next += 1;
        self.tiles[y][x] = Tile::Pressure(0);
        self.items[y][x] = None;
        self.flames[y][x] = None;
        self.bombs.retain(|b| !(b.grounded() && b.tile() == (x, y)));
        self.events.push(Event::Pressure { x, y });
        for j in 0..MAX_PLAYERS {
            if self.players[j].alive && self.players[j].tile() == (x, y) {
                self.die(j, None);
            }
        }
    }

    fn hurt_and_pickup(&mut self) {
        for i in 0..MAX_PLAYERS {
            let p = &self.players[i];
            if !p.joined || !p.alive {
                continue;
            }
            let (tx, ty) = p.tile();
            // a block crumbling round you (block pass) burns too, just as it catches
            let burning = matches!(self.tiles[ty][tx], Tile::Burning(t) if t > BURN - 8);
            if let Some(f) = self.flames[ty][tx] {
                self.hurt(i, Some(f.owner));
            } else if burning {
                self.hurt(i, None);
            }
            if !self.players[i].alive {
                continue;
            }
            if let Some(item) = self.items[ty][tx].take() {
                self.apply(i, item);
                self.events.push(Event::Pickup { player: i, item });
            }
            if let Some(what) = self.pickups[ty][tx].take() {
                if let Pickup::Key(c) = what {
                    self.players[i].keys[c as usize] += 1;
                }
                self.events.push(Event::Got { player: i, what });
            }
        }
    }

    fn apply(&mut self, i: usize, item: Item) {
        let p = &mut self.players[i];
        if item != Item::Skull {
            // any other item cures a disease
            p.disease = None;
        }
        if !matches!(item, Item::Skull | Item::Heart | Item::Vest) {
            p.picked.push(item);
        }
        p.items_got += 1;
        match item {
            Item::BombUp => p.bombs = (p.bombs + 1).min(MAX_BOMBS),
            Item::FireUp => p.fire = (p.fire + 1).min(MAX_FIRE),
            Item::FullFire => p.fire = MAX_FIRE,
            Item::SpeedUp => p.speed = (p.speed + 1).min(MAX_SPEED),
            Item::SpeedDown => p.speed = (p.speed - 1).max(MIN_SPEED),
            Item::Kick => p.kick = true,
            Item::Punch => p.punch = true,
            Item::Glove => p.glove = true,
            Item::LineBomb => p.line = true,
            Item::Remote => p.kind = BombKind::Remote,
            Item::Pierce => p.kind = BombKind::Pierce,
            Item::Rubber => p.kind = BombKind::Rubber,
            Item::Mine => p.kind = BombKind::Mine,
            Item::Surge => p.kind = BombKind::Surge,
            Item::BombPass => p.bomb_pass = true,
            Item::BlockPass => p.block_pass = true,
            Item::Heart => p.heart = true,
            Item::Vest => p.vest = VEST,
            Item::Skull => {
                let d = DISEASES[self.rand(DISEASES.len() as u32) as usize];
                let p = &mut self.players[i];
                p.disease = Some(d);
                p.disease_left = DISEASE;
                p.infect_cool = INFECT_COOL;
            }
        }
    }

    /// Diseases pass by touch: the carrier is cured, the other catches it.
    fn infect(&mut self) {
        for a in 0..MAX_PLAYERS {
            for b in 0..MAX_PLAYERS {
                let (pa, pb) = (&self.players[a], &self.players[b]);
                if a == b || !pa.alive || !pb.alive || pa.disease.is_none() || pa.infect_cool > 0 || pb.infect_cool > 0 {
                    continue;
                }
                if (pa.x - pb.x).abs() < TILE * 3 / 4 && (pa.y - pb.y).abs() < TILE * 3 / 4 {
                    let (d, left) = (pa.disease, pa.disease_left);
                    let pb = &mut self.players[b];
                    pb.disease = d;
                    pb.disease_left = left.max(DISEASE / 2);
                    pb.infect_cool = INFECT_COOL;
                    let pa = &mut self.players[a];
                    pa.disease = None;
                    pa.infect_cool = INFECT_COOL;
                    self.events.push(Event::Infect { from: a, to: b });
                }
            }
        }
    }

    pub(crate) fn hurt_pub(&mut self, i: usize, by: Option<usize>) {
        self.hurt(i, by);
    }

    fn hurt(&mut self, i: usize, by: Option<usize>) {
        let p = &mut self.players[i];
        if p.invuln > 0 || p.vest > 0 || p.hop.is_some() {
            return;
        }
        // a mount takes the hit for you (and the next egg hatches)
        if p.mount.is_some() {
            p.mount = if p.spare.is_empty() { None } else { Some(p.spare.remove(0)) };
            p.invuln = INVULN;
            p.dash = 0;
            self.events.push(Event::MountLost { player: i });
            return;
        }
        if p.heart {
            p.heart = false;
            p.invuln = INVULN;
            self.events.push(Event::Saved { player: i });
            return;
        }
        self.die(i, by);
    }

    fn die(&mut self, i: usize, by: Option<usize>) {
        let (tx, ty) = self.players[i].tile();
        let p = &mut self.players[i];
        p.alive = false;
        p.deaths += 1;
        p.disease = None;
        p.stun = 0;
        self.events.push(Event::Death { player: i, by });
        if let Some(k) = by
            && k != i
        {
            self.players[k].kills += 1;
        }
        // whatever you were carrying goes off where you fell
        if let Some(k) = self.bombs.iter().position(|b| b.motion == Motion::Carried { by: i }) {
            let b = &mut self.bombs[k];
            b.motion = Motion::Still;
            (b.x, b.y) = (centre(tx), centre(ty));
            b.fuse = 1;
        }
        if self.adventure {
            // handling items go with a life, as in SB3
            let p = &mut self.players[i];
            p.kick = false;
            p.punch = false;
            p.glove = false;
            p.line = false;
            p.kind = BombKind::Normal;
            p.bomb_pass = false;
            p.block_pass = false;
            p.mount = None;
            p.spare.clear();
            p.hop = None;
            p.dash = 0;
        }
        if !self.battle {
            self.players[i].respawn = RESPAWN;
            return;
        }
        self.players[i].out = true;
        let picked = std::mem::take(&mut self.players[i].picked);
        self.scatter(&picked);
        if self.revenge && !self.hurry() {
            self.players[i].rider = Some(self.nearest_ring(tx, ty));
        }
        // a rider's bomb that knocks someone out brings the rider back in their place
        if let Some(k) = by
            && k != i
            && self.comeback
            && self.players[k].rider.is_some()
        {
            let p = &mut self.players[k];
            let rejoined = Player { joined: true, x: centre(tx), y: centre(ty), alive: true, invuln: INVULN, deaths: p.deaths, ..Player::new() };
            *p = rejoined;
            self.events.push(Event::Comeback { player: k });
        }
    }

    /// Scatter a knocked-out player's items onto free floor.
    fn scatter(&mut self, items: &[Item]) {
        let mut free: Vec<(usize, usize)> = Vec::new();
        for y in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                if self.tiles[y][x] == Tile::Floor && self.items[y][x].is_none() && self.flames[y][x].is_none() && self.bomb_at(x, y).is_none() {
                    free.push((x, y));
                }
            }
        }
        for &item in items {
            if free.is_empty() {
                break;
            }
            let k = self.rand(free.len() as u32) as usize;
            let (x, y) = free.swap_remove(k);
            self.items[y][x] = Some(item);
        }
    }

    fn nearest_ring(&self, x: usize, y: usize) -> i32 {
        let r = ring(self.w, self.h);
        let k = (0..r.len()).min_by_key(|&k| r[k].0.abs_diff(x) + r[k].1.abs_diff(y)).unwrap_or(0);
        k as i32 * TILE
    }

    /// Revenge riders: slide along the border and lob bombs in, one at a time.
    fn ride(&mut self, i: usize, c: Cmd) {
        let Some(pos) = self.players[i].rider else { return };
        let r = ring(self.w, self.h);
        let n = r.len() as i32;
        let k = (pos + TILE / 2).div_euclid(TILE).rem_euclid(n) as usize;
        let (tx, ty) = r[k];
        let (w, h) = (self.w, self.h);
        // which way is clockwise on this side: top goes right, right goes down, ...
        let cw = |d: Dir| -> i32 {
            let mut v: i32 = 0;
            if ty == 0 {
                v += match d { Dir::Right => 1, Dir::Left => -1, _ => 0 };
            }
            if tx == w - 1 {
                v += match d { Dir::Down => 1, Dir::Up => -1, _ => 0 };
            }
            if ty == h - 1 {
                v += match d { Dir::Left => 1, Dir::Right => -1, _ => 0 };
            }
            if tx == 0 {
                v += match d { Dir::Up => 1, Dir::Down => -1, _ => 0 };
            }
            v.signum()
        };
        let mut delta = c.primary.map_or(0, cw);
        if delta == 0 {
            delta = c.secondary.map_or(0, cw);
        }
        if delta != 0 {
            self.players[i].rider = Some((pos + delta * BASE_SPEED).rem_euclid(n * TILE));
            self.players[i].walk += 1;
        } else {
            self.players[i].walk = 0;
        }
        if c.bomb && !self.bombs.iter().any(|b| b.owner == i && b.from_rider) {
            let (dx, dy) = if ty == 0 {
                (0, 1)
            } else if ty == self.h - 1 {
                (0, -1)
            } else if tx == 0 {
                (1, 0)
            } else {
                (-1, 0)
            };
            let (sx, sy) = ((tx as i32).clamp(1, self.w as i32 - 2), (ty as i32).clamp(1, self.h as i32 - 2));
            let to = (sx + dx * (THROW_TILES - 1), sy + dy * (THROW_TILES - 1));
            let mut b = self.new_bomb(i, tx, ty, RIDER_FIRE, BombKind::Normal, FUSE);
            b.from_rider = true;
            b.pass = [false; MAX_PLAYERS];
            self.bombs.push(b);
            let k = self.bombs.len() - 1;
            self.launch(k, rider_pos(pos, self.w, self.h), to, dx, dy, THROW_TILES as u32 * FLY_STEPS_PER_TILE);
            self.events.push(Event::Throw { player: i });
        }
    }

    /// Players still in the round (battle).
    pub fn standing(&self) -> Vec<usize> {
        (0..MAX_PLAYERS).filter(|&i| self.players[i].joined && self.players[i].alive && !self.players[i].out).collect()
    }

    /// A checksum of the whole state, for determinism tests and netplay desync checks.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |v: u64| {
            h ^= v;
            h = h.wrapping_mul(0x0100_0000_01b3);
        };
        for p in &self.players {
            mix(p.x as u64);
            mix(p.y as u64);
            mix(p.alive as u64);
            mix(p.rider.unwrap_or(-1) as u64);
        }
        for b in &self.bombs {
            mix(b.x as u64 ^ (b.y as u64) << 20 ^ (b.fuse as u64) << 40);
        }
        for row in &self.tiles {
            for t in row {
                mix(match t {
                    Tile::Floor => 0,
                    Tile::Wall => 1,
                    Tile::Soft => 2,
                    Tile::Burning(n) => 3 + *n as u64,
                    Tile::Pressure(n) => 100 + *n as u64,
                    Tile::Dud => 200,
                    Tile::Block => 300,
                    Tile::Door(c) => 400 + *c as u64,
                    Tile::Gate => 500,
                    Tile::Socket => 600,
                });
            }
        }
        mix(self.rng);
        h
    }
}

impl Game {
    /// Wrap a landing tile round the arena's interior, as thrown bombs do.
    fn wrap(&self, (x, y): (i32, i32)) -> (i32, i32) {
        let wx = |v: i32, n: i32| if v < 1 { v + (n - 2) } else if v > n - 2 { v - (n - 2) } else { v };
        (wx(x, self.w as i32), wx(y, self.h as i32))
    }

    /// Spawn corners and the two tiles next to each are always clear.
    fn near_spawn(&self, x: usize, y: usize) -> bool {
        self.spawns.iter().any(|&(sx, sy)| x.abs_diff(sx) + y.abs_diff(sy) <= 1)
    }

    /// A revenge rider's position, in units.
    pub fn rider_pos(&self, p: i32) -> (i32, i32) {
        rider_pos(p, self.w, self.h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(seed: u64) -> Game {
        let mut g = Game::new(seed);
        for y in 0..H {
            for x in 0..W {
                if g.tiles[y][x] == Tile::Soft {
                    g.tiles[y][x] = Tile::Floor;
                }
            }
        }
        g.join(0);
        g
    }

    fn go(d: Dir) -> [Cmd; MAX_PLAYERS] {
        let mut c = [Cmd::default(); MAX_PLAYERS];
        c[0].primary = Some(d);
        c
    }

    fn idle() -> [Cmd; MAX_PLAYERS] {
        [Cmd::default(); MAX_PLAYERS]
    }

    fn act() -> [Cmd; MAX_PLAYERS] {
        let mut c = idle();
        c[0].action = true;
        c
    }

    fn bomb() -> [Cmd; MAX_PLAYERS] {
        let mut c = idle();
        c[0].bomb = true;
        c
    }

    #[test]
    fn walks_a_corridor() {
        let mut g = open(1);
        for _ in 0..60 {
            g.step(&go(Dir::Right));
        }
        assert!(g.players[0].tile().0 >= 3, "{:?}", g.players[0].tile());
        assert_eq!(g.players[0].y, centre(1));
    }

    #[test]
    fn lines_up_with_the_lane() {
        let mut g = open(1);
        g.players[0].y += 40;
        for _ in 0..21 {
            g.step(&go(Dir::Right));
        }
        assert_eq!(g.players[0].y, centre(1));
    }

    #[test]
    fn corner_assist_slides_round_a_pillar() {
        // stand in (2,1) shifted toward column 3 and go down: (2,2) is a pillar and
        // (3,2) is open, so assist slides right into column 3 and on down
        let mut g = open(1);
        g.players[0].x = centre(2) + 80;
        g.players[0].y = centre(1);
        for _ in 0..30 {
            g.step(&go(Dir::Down));
        }
        let p = &g.players[0];
        assert_eq!(p.x, centre(3));
        assert!(p.y > centre(1), "should be heading down, at {}", p.y);
    }

    #[test]
    fn blocked_square_on_stops() {
        let mut g = open(1);
        g.players[0].x = centre(2);
        let y0 = g.players[0].y;
        for _ in 0..30 {
            g.step(&go(Dir::Down));
        }
        assert_eq!(g.players[0].y, y0);
    }

    #[test]
    fn cannot_walk_back_onto_own_bomb() {
        let mut g = open(1);
        g.step(&bomb());
        assert_eq!(g.bombs.len(), 1);
        for _ in 0..40 {
            g.step(&go(Dir::Right));
        }
        for _ in 0..40 {
            g.step(&go(Dir::Left));
        }
        assert!(g.players[0].x >= centre(2), "walked back onto the bomb: {}", g.players[0].x);
    }

    #[test]
    fn chain_reaction_same_step() {
        let mut g = open(1);
        g.players[0].fire = 2;
        g.place_bomb(0, 5, 1);
        g.place_bomb(0, 7, 1);
        g.bombs[1].fuse = 999;
        g.bombs[0].fuse = 1;
        g.step(&idle());
        assert!(g.bombs.is_empty(), "second bomb should go off in the same step");
        assert!(g.flames[1][9].is_some());
    }

    #[test]
    fn blast_stops_at_soft_block_and_breaks_it() {
        let mut g = open(1);
        g.tiles[1][7] = Tile::Soft;
        g.players[0].fire = 4;
        g.place_bomb(0, 5, 1);
        g.bombs[0].fuse = 1;
        g.step(&idle());
        assert!(matches!(g.tiles[1][7], Tile::Burning(_)));
        assert!(g.flames[1][8].is_none());
    }

    #[test]
    fn pierce_goes_through_soft_blocks() {
        let mut g = open(1);
        g.tiles[1][7] = Tile::Soft;
        g.players[0].fire = 4;
        g.players[0].kind = BombKind::Pierce;
        g.place_bomb(0, 5, 1);
        g.bombs[0].fuse = 1;
        g.step(&idle());
        assert!(matches!(g.tiles[1][7], Tile::Burning(_)));
        assert!(g.flames[1][8].is_some());
    }

    /// A walled room with nothing inside.
    fn room(seed: u64) -> Game {
        let mut g = open(seed);
        for y in 1..H - 1 {
            for x in 1..W - 1 {
                g.tiles[y][x] = Tile::Floor;
            }
        }
        g
    }

    fn surge_at(g: &mut Game, x: usize, y: usize) {
        g.players[0].kind = BombKind::Surge;
        g.place_bomb(0, x, y);
        g.bombs[0].fuse = 1;
    }

    #[test]
    fn surge_floods_a_diamond_in_the_open() {
        let mut g = room(1);
        g.players[0].fire = 4;
        g.players[0].x = centre(1);
        g.players[0].y = centre(1);
        g.players[0].vest = 9999;
        surge_at(&mut g, 7, 6);
        for _ in 0..(SURGE_RING * 4) {
            g.step(&idle());
        }
        let reach = surge_reach(4) as i32;
        for y in 1..H - 1 {
            for x in 1..W - 1 {
                let d = (x as i32 - 7).abs() + (y as i32 - 6).abs();
                assert_eq!(g.flames[y][x].is_some(), d <= reach, "tile {x},{y} at distance {d}");
            }
        }
        assert!(g.surges.is_empty());
    }

    #[test]
    fn surge_spreads_a_ring_at_a_time() {
        let mut g = room(1);
        g.players[0].fire = 8;
        g.players[0].vest = 9999;
        surge_at(&mut g, 7, 6);
        g.step(&idle());
        g.step(&idle());
        assert!(g.flames[6][8].is_some());
        assert!(g.flames[6][9].is_none(), "the second ring should still be on its way");
    }

    #[test]
    fn surge_flows_round_a_wall_and_stops_at_a_soft_block() {
        let mut g = room(1);
        g.players[0].fire = 8;
        g.players[0].vest = 9999;
        g.tiles[6][8] = Tile::Wall;
        g.tiles[5][7] = Tile::Soft;
        surge_at(&mut g, 7, 6);
        for _ in 0..(SURGE_RING * 6) {
            g.step(&idle());
        }
        // behind the wall, reached the long way round
        assert!(g.flames[6][9].is_some());
        assert!(matches!(g.tiles[5][7], Tile::Burning(_)));
    }

    #[test]
    fn surge_sets_off_a_bomb_it_reaches() {
        let mut g = room(1);
        g.players[0].fire = 4;
        g.players[0].vest = 9999;
        g.players[0].bombs = 2;
        surge_at(&mut g, 7, 6);
        g.place_bomb(0, 9, 7);
        g.bombs[1].fuse = 999;
        for _ in 0..(SURGE_RING * 4) {
            g.step(&idle());
        }
        assert!(g.bombs.is_empty(), "the bomb two tiles off the diagonal should have gone off");
    }

    /// A room with player 1 standing on (x, y).
    fn stand(x: usize, y: usize) -> Game {
        let mut g = room(1);
        g.players[0].x = centre(x);
        g.players[0].y = centre(y);
        g
    }

    fn walk_for(g: &mut Game, d: Dir, steps: u32) {
        for _ in 0..steps {
            g.step(&go(d));
        }
    }

    #[test]
    fn leaning_on_a_block_pushes_it() {
        let mut g = stand(5, 6);
        g.tiles[6][6] = Tile::Block;
        walk_for(&mut g, Dir::Right, PUSH_DELAY + 1);
        assert_eq!(g.tiles[6][7], Tile::Block);
        assert_eq!(g.tiles[6][6], Tile::Floor);
        walk_for(&mut g, Dir::Right, SHOVE_STEPS + 30);
        assert!(g.players[0].x > centre(6), "should follow the block: {}", g.players[0].x);
    }

    #[test]
    fn brushing_a_block_does_not_push_it() {
        let mut g = stand(5, 6);
        g.tiles[6][6] = Tile::Block;
        walk_for(&mut g, Dir::Right, PUSH_DELAY / 2);
        g.step(&idle());
        walk_for(&mut g, Dir::Right, PUSH_DELAY / 2);
        assert_eq!(g.tiles[6][6], Tile::Block);
    }

    #[test]
    fn a_block_stops_at_a_wall_and_at_a_bomb() {
        let mut g = stand(5, 6);
        g.tiles[6][6] = Tile::Block;
        g.tiles[6][7] = Tile::Wall;
        g.tiles[4][5] = Tile::Block;
        g.place_bomb(0, 5, 3);
        g.bombs[0].fuse = 999;
        walk_for(&mut g, Dir::Right, 40);
        assert_eq!(g.tiles[6][6], Tile::Block);
        g.players[0].x = centre(5);
        walk_for(&mut g, Dir::Up, 40);
        assert_eq!(g.tiles[4][5], Tile::Block);
    }

    #[test]
    fn a_block_pushed_into_water_makes_floor() {
        let mut g = stand(5, 6);
        g.tiles[6][6] = Tile::Block;
        g.tiles[6][7] = Tile::Wall;
        g.ground[6][7] = Ground::Water;
        walk_for(&mut g, Dir::Right, PUSH_DELAY + SHOVE_STEPS + 2);
        assert_eq!((g.tiles[6][7], g.ground[6][7]), (Tile::Floor, Ground::Sunk));
        walk_for(&mut g, Dir::Right, 60);
        assert!(g.players[0].x > centre(7), "should walk over the sunk block");
    }

    #[test]
    fn a_block_slides_across_ice() {
        let mut g = stand(5, 6);
        g.tiles[6][6] = Tile::Block;
        for x in 7..10 {
            g.ground[6][x] = Ground::Ice;
        }
        walk_for(&mut g, Dir::Right, PUSH_DELAY + 1);
        for _ in 0..(SHOVE_STEPS * 5) {
            g.step(&idle());
        }
        assert_eq!(g.tiles[6][10], Tile::Block, "should slide off the ice onto plain floor");
    }

    #[test]
    fn a_belt_carries_a_block() {
        let mut g = stand(1, 1);
        g.tiles[6][6] = Tile::Block;
        g.ground[6][6] = Ground::Belt(Dir::Down);
        for _ in 0..(SHOVE_STEPS + 2) {
            g.step(&idle());
        }
        assert_eq!(g.tiles[7][6], Tile::Block);
    }

    #[test]
    fn a_blast_stops_at_a_block_and_leaves_it() {
        let mut g = stand(1, 1);
        g.tiles[6][7] = Tile::Block;
        g.players[0].fire = 4;
        g.place_bomb(0, 5, 6);
        g.bombs[0].fuse = 1;
        g.step(&idle());
        assert_eq!(g.tiles[6][7], Tile::Block);
        assert!(g.flames[6][6].is_some() && g.flames[6][8].is_none());
    }

    #[test]
    fn a_key_opens_its_door_only() {
        let mut g = stand(5, 6);
        g.pickups[6][6] = Some(Pickup::Key(1));
        g.tiles[6][8] = Tile::Door(1);
        g.tiles[6][10] = Tile::Door(2);
        walk_for(&mut g, Dir::Right, 120);
        assert_eq!(g.tiles[6][8], Tile::Floor);
        assert_eq!(g.tiles[6][10], Tile::Door(2));
        assert_eq!(g.players[0].keys, [0; 4]);
    }

    #[test]
    fn gates_open_while_every_switch_is_held() {
        let mut g = stand(3, 6);
        g.ground[6][3] = Ground::Switch;
        g.ground[2][9] = Ground::Switch;
        g.tiles[10][10] = Tile::Gate;
        g.ground[10][10] = Ground::Gate;
        g.step(&idle());
        assert_eq!(g.tiles[10][10], Tile::Gate, "one switch of two is not enough");
        g.tiles[2][9] = Tile::Block;
        g.step(&idle());
        assert_eq!(g.tiles[10][10], Tile::Floor);
        walk_for(&mut g, Dir::Left, 40);
        assert_eq!(g.tiles[10][10], Tile::Gate, "stepping off shuts it again");
    }

    #[test]
    fn the_socket_opens_once_every_part_is_found() {
        let mut g = stand(5, 6);
        g.pickups[6][6] = Some(Pickup::Part);
        g.pickups[6][8] = Some(Pickup::Part);
        g.tiles[10][10] = Tile::Socket;
        walk_for(&mut g, Dir::Right, 40);
        assert_eq!(g.tiles[10][10], Tile::Socket);
        walk_for(&mut g, Dir::Right, 40);
        assert_eq!(g.tiles[10][10], Tile::Floor);
    }

    /// A bomb on (x, y) whose fuse runs out next step and that is sure to fizzle.
    fn fizzling(g: &mut Game, x: usize, y: usize) {
        g.fizzle = 100;
        g.place_bomb(0, x, y);
        g.bombs[0].fuse = 1;
        g.step(&idle());
    }

    #[test]
    fn a_fizzled_bomb_lies_live_and_frees_its_owner() {
        let mut g = stand(1, 1);
        fizzling(&mut g, 7, 6);
        assert!(g.bombs[0].uxo.is_some(), "should have fizzled");
        assert!(g.flames[6][7].is_none());
        assert_eq!(g.owned(0), 0, "a UXO no longer counts against your bombs");
    }

    #[test]
    fn a_uxo_goes_off_by_itself_in_time() {
        let mut g = stand(1, 1);
        fizzling(&mut g, 7, 6);
        for _ in 0..(UXO_MIN + UXO_SPAN + 1) {
            g.step(&idle());
        }
        assert!(g.bombs.is_empty(), "it should have gone off by now");
    }

    #[test]
    fn a_blast_sets_off_a_uxo() {
        let mut g = stand(1, 1);
        fizzling(&mut g, 7, 6);
        g.fizzle = 0;
        g.players[0].bombs = 2;
        g.players[0].fire = 3;
        g.place_bomb(0, 5, 6);
        g.bombs[1].fuse = 1;
        g.step(&idle());
        assert!(g.bombs.is_empty());
        assert!(g.flames[6][9].is_some(), "the UXO's own blast");
    }

    #[test]
    fn a_kicked_uxo_goes_off_where_it_stops() {
        let mut g = stand(4, 6);
        fizzling(&mut g, 6, 6);
        g.players[0].kick = true;
        g.players[0].vest = 9999;
        // about 22 steps to reach it, then 45 or so to slide seven tiles
        walk_for(&mut g, Dir::Right, 30);
        assert!(g.bombs[0].uxo.is_some(), "still live while it slides");
        walk_for(&mut g, Dir::Right, 60);
        assert!(g.bombs.is_empty(), "should have gone off against the wall");
        assert!(g.flames[6][13].is_some());
    }

    #[test]
    fn the_duds_disease_makes_every_bomb_fizzle() {
        let mut g = stand(1, 1);
        g.players[0].disease = Some(Disease::Duds);
        g.players[0].disease_left = 9999;
        g.place_bomb(0, 7, 6);
        g.bombs[0].fuse = 1;
        g.step(&idle());
        assert!(g.bombs[0].uxo.is_some());
    }

    #[test]
    fn a_charge_goes_off_with_a_real_blast() {
        let mut g = stand(1, 1);
        g.tiles[6][7] = Tile::Dud;
        g.players[0].fire = 2;
        g.place_bomb(0, 5, 6);
        g.bombs[0].fuse = 1;
        g.step(&idle());
        assert_eq!(g.tiles[6][7], Tile::Floor);
        assert!(g.flames[6][7 + CHARGE_FIRE as usize].is_some(), "the charge's own blast reaches {CHARGE_FIRE}");
        assert!(g.flames[6 - CHARGE_FIRE as usize][7].is_some());
    }

    #[test]
    fn kicked_bomb_slides_to_the_wall() {
        let mut g = open(1);
        g.players[0].kick = true;
        g.place_bomb(0, 3, 1);
        g.bombs[0].fuse = 999;
        // about 22 steps to reach the bomb, then 64 for it to cross ten tiles
        for _ in 0..120 {
            g.step(&go(Dir::Right));
        }
        let b = &g.bombs[0];
        assert_eq!(b.motion, Motion::Still);
        assert_eq!(b.tile(), (13, 1), "should stop against the east wall");
    }

    #[test]
    fn rubber_bomb_bounces_back() {
        let mut g = open(1);
        g.players[0].kick = true;
        g.players[0].kind = BombKind::Rubber;
        g.players[0].x = centre(7);
        g.place_bomb(0, 9, 1);
        g.bombs[0].fuse = 999;
        g.bombs[0].kind = BombKind::Rubber;
        g.players[0].x = centre(8);
        for _ in 0..6 {
            g.step(&go(Dir::Right));
        }
        // four tiles to the east wall takes about 26 steps; then it comes back
        for _ in 0..40 {
            g.step(&idle());
        }
        assert!(matches!(g.bombs[0].motion, Motion::Slide { dx: -1, .. }), "{:?}", g.bombs[0].motion);
        assert!(g.bombs[0].tile().0 < 13, "should be on its way back west");
    }

    #[test]
    fn punch_sends_a_bomb_three_tiles() {
        let mut g = open(1);
        g.players[0].punch = true;
        g.players[0].facing = Dir::Right;
        g.place_bomb(0, 2, 1);
        g.bombs[0].fuse = 999;
        g.step(&act());
        for _ in 0..30 {
            g.step(&idle());
        }
        assert_eq!(g.bombs[0].tile(), (5, 1));
        assert_eq!(g.bombs[0].motion, Motion::Still);
    }

    #[test]
    fn glove_picks_up_and_throws() {
        let mut g = open(1);
        g.players[0].glove = true;
        g.players[0].facing = Dir::Right;
        g.step(&bomb());
        g.step(&act());
        assert!(matches!(g.bombs[0].motion, Motion::Carried { by: 0 }));
        g.step(&act());
        for _ in 0..30 {
            g.step(&idle());
        }
        assert_eq!(g.bombs[0].tile(), (4, 1));
    }

    #[test]
    fn remote_waits_for_the_button() {
        let mut g = open(1);
        g.players[0].kind = BombKind::Remote;
        g.step(&bomb());
        for _ in 0..40 {
            g.step(&go(Dir::Right));
        }
        for _ in 0..(FUSE * 2) {
            g.step(&idle());
        }
        assert_eq!(g.bombs.len(), 1, "remote bomb went off on its own");
        g.step(&act());
        assert!(g.bombs.is_empty());
    }

    #[test]
    fn heart_takes_one_hit() {
        let mut g = open(1);
        g.battle = true;
        g.players[0].heart = true;
        g.flames[1][1] = Some(Flame { left: 5, links: 0, owner: 0, surge: false });
        g.step(&idle());
        assert!(g.players[0].alive);
        assert!(!g.players[0].heart);
    }

    #[test]
    fn battle_knockout_becomes_a_rider_and_lobs() {
        let mut g = open(1);
        g.battle = true;
        g.join(1);
        g.flames[1][1] = Some(Flame { left: 5, links: 0, owner: 1, surge: false });
        g.step(&idle());
        assert!(g.players[0].out && g.players[0].rider.is_some());
        g.step(&bomb());
        assert!(g.bombs.iter().any(|b| b.from_rider && matches!(b.motion, Motion::Fly { .. })));
        assert_eq!(g.standing(), vec![1]);
    }

    #[test]
    fn rider_kill_brings_the_rider_back() {
        let mut g = open(1);
        g.battle = true;
        g.join(1);
        g.players[0].out = true;
        g.players[0].alive = false;
        g.players[0].rider = Some(0);
        let (x, y) = g.players[1].tile();
        g.flames[y][x] = Some(Flame { left: 5, links: 0, owner: 0, surge: false });
        g.step(&idle());
        assert!(g.players[0].alive && !g.players[0].out, "rider should be back");
        assert!(g.players[1].out);
    }

    #[test]
    fn sudden_death_fills_and_crushes() {
        let mut g = open(1);
        g.battle = true;
        g.clock = HURRY_AT - 1;
        g.step(&idle());
        assert!(matches!(g.tiles[1][1], Tile::Pressure(_)));
        assert!(!g.players[0].alive, "the block landed on the player at (1,1)");
    }

    #[test]
    fn disease_passes_by_touch() {
        let mut g = open(1);
        g.join(1);
        g.players[1].x = g.players[0].x + 40;
        g.players[1].y = g.players[0].y;
        g.players[0].disease = Some(Disease::Reverse);
        g.players[0].disease_left = 500;
        g.step(&idle());
        assert_eq!(g.players[0].disease, None);
        assert_eq!(g.players[1].disease, Some(Disease::Reverse));
    }

    #[test]
    fn mine_waits_for_a_step() {
        let mut g = open(1);
        g.join(1);
        g.players[0].kind = BombKind::Mine;
        g.place_bomb(0, 3, 1);
        for _ in 0..(FUSE * 2) {
            g.step(&idle());
        }
        assert_eq!(g.bombs.len(), 1);
        assert!(g.bombs[0].armed);
        g.players[1].x = centre(3);
        g.players[1].y = centre(1);
        for _ in 0..(MINE_TRIGGER + 2) {
            g.step(&idle());
        }
        assert!(g.bombs.is_empty(), "stepping on the mine should set it off");
    }

    #[test]
    fn line_bomb_lays_a_row() {
        let mut g = open(1);
        g.players[0].line = true;
        g.players[0].bombs = 4;
        g.players[0].facing = Dir::Right;
        g.step(&bomb());
        g.step(&bomb());
        assert_eq!(g.bombs.len(), 4);
        let mut xs: Vec<usize> = g.bombs.iter().map(|b| b.tile().0).collect();
        xs.sort();
        assert_eq!(xs, vec![1, 2, 3, 4]);
    }

    #[test]
    fn belt_carries_a_player() {
        let mut g = open(1);
        for x in 1..6 {
            g.ground[1][x] = Ground::Belt(Dir::Right);
        }
        for _ in 0..60 {
            g.step(&idle());
        }
        assert!(g.players[0].x > centre(2), "the belt should have moved the player right");
    }

    #[test]
    fn arrow_turns_a_kicked_bomb() {
        let mut g = open(1);
        g.players[0].kick = true;
        g.ground[1][5] = Ground::Arrow(Dir::Down);
        g.place_bomb(0, 3, 1);
        g.bombs[0].fuse = 999;
        for _ in 0..120 {
            g.step(&go(Dir::Right));
        }
        let b = &g.bombs[0];
        assert_eq!(b.tile().0, 5, "turned down at the arrow");
        assert!(b.tile().1 > 1);
    }

    #[test]
    fn warp_sends_you_to_the_next_one() {
        let mut g = open(1);
        g.ground[1][2] = Ground::Warp;
        g.ground[11][13] = Ground::Warp;
        for _ in 0..30 {
            g.step(&go(Dir::Right));
        }
        assert_eq!(g.players[0].tile(), (13, 11));
        assert!(g.events.iter().any(|e| matches!(e, Event::Warp { player: 0 })) || g.players[0].warped);
    }

    #[test]
    fn ice_keeps_you_sliding() {
        let mut g = open(1);
        for x in 2..13 {
            g.ground[1][x] = Ground::Ice;
        }
        for _ in 0..25 {
            g.step(&go(Dir::Right));
        }
        let x0 = g.players[0].x;
        for _ in 0..30 {
            g.step(&idle());
        }
        assert!(g.players[0].x > x0 + TILE, "let go of the stick on ice and you keep going");
    }

    #[test]
    fn deterministic() {
        let run = || {
            let mut g = Game::new(42);
            g.battle = true;
            g.join(0);
            g.join(1);
            for s in 0..900u32 {
                let mut c = [Cmd::default(); MAX_PLAYERS];
                c[0].primary = Some([Dir::Right, Dir::Down, Dir::Left, Dir::Up][(s / 37 % 4) as usize]);
                c[0].bomb = s % 90 == 0;
                c[0].action = s % 50 == 0;
                c[1].primary = Some([Dir::Up, Dir::Left][(s / 50 % 2) as usize]);
                c[1].bomb = s % 120 == 5;
                g.step(&c);
            }
            g.hash()
        };
        assert_eq!(run(), run());
    }
}
