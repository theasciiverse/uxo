# uXo

Asciiverse bomb-arena action game. Rust + Ratatui (+ half-blocks for tile art).
A tribute to the 90s bomb-arena games, Super Bomberman 3 to 5 above all: the grid,
the cross-shaped blasts, the chain reactions, and above all the big power-up set
that made those games feel like chaos you could still read. Same kind of
mechanics, our own identity. Not a clone.

**Status:** World 1 of Adventure playable, battle mode complete (2026-10-04).
A title screen (big pixel logo over fireworks) leads to **Adventure** (alone or two
together: the Orchard's nine stages, story cards, six baddie species and the
Bramblehog boss, duds, tamed mounts, lives, score, a clock, saved progress for
Continue) or **Battle**.

Battle: One player practises; a second
joining opens the battle setup (stage or all in turn, wins to take the match,
revenge off / ride / ride and come back, sudden death, item frequency). Six original
battle maps (25x17, bigger than the view) with gimmicks and a camera that follows
the players and zooms to keep everyone in view; rounds with revenge riders and
sudden death; the full item set (18 items, skull diseases) with kick, punch, glove,
line bomb, remote, pierce, rubber and mine bombs. A pixel-font top bar says where you
are (stage, round, clock); player cards at the sides (hero, wins, knockouts, blocks,
item badges) when the terminal is wide; a results table after each round and awards
after the match. Every gamepad is its own player. Next: Adventure (see Build order).

Controls: move with the stick, d-pad, WASD or arrows; **bomb** A or X, Space
(WASD player) or Enter (arrows player); **ride** (your mount's hop or dash) Y, E
(WASD) or `.` (arrows); **action** B, F (WASD) or `/` (arrows): throw what you carry, else punch the bomb ahead, else pick up the bomb
underfoot, else set off your oldest remote bomb. Pressing bomb again while standing
on your own bomb with the line bomb lays the rest in a row. Menus: direction taps
choose and change, bomb confirms. Start or P pauses, `+`/`-` size (`0` auto), Esc or
a pad's Select goes back to the title (and quits from there; Q and R were dropped:
they sit next to WASD).

```bash
cargo run --release                           # play; +/- switches size
RUST_LOG=debug cargo run --release 2>uxo.log  # with the session log
cargo run --release -- --snapshot out.ppm --zoom l --size 100x42  # staged scene as an image (zoom s, m or l)
cargo test --release                          # movement, assist, chains, determinism
```

## Name

**uXo**, chosen by David on 2026-10-04. UXO is unexploded ordnance, which is
exactly what every ticking bomb on the grid is. The capital X in the middle reads
as the blast itself going off between two round shapes, so the name is already a
logo; in the terminal title the X can be the one glyph that flashes.

- **uXo**: the logo, in-game and on the store page.
- **UXO**: plain text (search, trademark filings, running prose).
- **`uxo`**: folder, crate and repo name. Free on crates.io; no public
  `theasciiverse/uxo` repo (checked 2026-10-04).
- Tone: UXO is also a real and deadly legacy (Laos, Vietnam, WWII bombs still
  turning up across Europe). Keep the hero, store copy and art cartoonish and
  playful, never military. That suits the SB tone anyway.

Other games called UXO: only *UXO: Unexploded Ordnance* (Neodolphino Productions,
2016), a Minesweeper-style NES homebrew cartridge. Different genre and platform;
don't use the "Unexploded Ordnance" subtitle, so the two stay distinct. Nothing on
Steam or itch.io. Checked 2026-10-04.

Rejected: "Fork Bomb", the perfect terminal pun, taken by *FORK BOMB: A
Programming Game* (Quiet Machines, Steam Early Access 2026-10-02).

## Rights

Checked 2026-10-04, not legal advice; a lawyer look at the final name and store
copy is cheap insurance before anything is sold.

- **Mechanics are free.** US: *Tetris Holding v. Xio* (2012) held rules and game
  mechanics are not protected by copyright; Xio lost because it copied Tetris's
  whole look and feel, not its rules. EU: *SAS Institute v. World Programming*
  (CJEU Grand Chamber, C-406/10, 2012): a program's functionality is not a form of
  expression and is not protected.
- **The name is not.** BOMBERMAN is a registered trademark of Konami Digital
  Entertainment (via its Hudson Soft acquisition). Don't use it in the title,
  store blurb or marketing. "A love letter to the 90s bomb-arena games" says it.
- **UXO trademark search** (TMview, 2026-10-04: EUIPO, USPTO, UK, Swedish PRV
  and most national offices; top 300 of 11,851 fuzzy hits read, exact matches
  first). No live UXO mark for games anywhere: nothing in class 28, nothing for
  video games in 41, and the only US software mark (Fuegotech, 2001) is dead.
  Live marks in nearby classes, all different markets:
  - Romania, class 9 + 35/36/38 (2021): downloadable software for a payments and
    insurance service.
  - EU, filed 2026-08-31 by "PAY BY UXO", classes 9/35/36/38, pending: very likely
    the same payments business going EU-wide. Goods list not retrievable. **Watch
    this one**; if it is granted with broad software wording, get a lawyer's view
    before selling in the EU.
  - China, class 9 (2020), goods not retrievable.
  - France, class 41 (2017): a publisher (books, films, training), no games.
  - The rest: shoes and clothing, wine, jewellery, sweets, advertising.
- **Genre precedent:** XBlast ("inspired by Dynablaster") since the 90s, and the
  commercial *Blast Zone! Tournament* on Steam.

So, ours and only ours: the name, the hero, the mounts and their designs, item
names and icons, music and sound, level layouts, bosses. Free to use: everything
in "Core rules" and "Power-ups" below as mechanics. The terminal itself keeps us
far from Hudson's 16-bit look, which removes most of the Tetris-case risk.

## Core rules (the genre, all free)

- Grid arena; indestructible pillars on a checker, soft blocks filling the rest
  with clear corners for spawns.
- Drop a bomb on your cell; it bursts after a fuse into a cross whose arm length
  is your fire. Arms stop at the first block (soft blocks break) unless pierced.
- Blasts set off other bombs: chain reactions are the heart of the game.
- Soft blocks hide items; blasts also destroy items lying in the open.
- Movement is smooth inside the grid, not cell-to-cell hops, with **corner
  assist** (pressing into a pillar edge slides you round it). This is what makes
  SB feel good and what most clones get wrong; it must be right from day one.
- You can't walk through bombs (unless you have the item) but you can step off
  one you just dropped.

## Power-ups

Mechanics only; every name and icon below is a placeholder for our own.

**Stat items:** extra bomb, fire up, full fire, speed up.
**Bomb handling:** kick, punch (knock a bomb over blocks), glove (pick up and
throw), line bomb (drop all held bombs in a row ahead of you), remote detonation.
**Bomb types:** pierce, bouncing (rubber), land mine, power bomb (max fire),
probably sticky or magnetic later. One special bomb type held at a time.
**Defence:** shield/vest (a few seconds of blast immunity), heart (absorb one hit).
**Walk-through:** bomb pass, soft-block pass.
**Diseases** (the skull): slow, too fast, short fuse, long fuse, tiny fire, no
bombs, auto-drop, reversed controls, invisible. Touching another player passes it
on. Good inspiration for more: Bombic 2's "infect everyone else", XBlast's list.

**Mounts:** the SB3-5 signature (eggs that hatch a rideable creature, each with an
ability: dash, hop over blocks, kick, line of bombs, punch). A hit costs the mount
instead of your life. Ours must be our own creatures, not kangaroos. This is the
place to be most original; nothing in the references designs it well except the
SB6 fan project's model of the rules.

## Modes

- **Adventure** (single player or co-op): the heart of the game, see below.
- **Battle**: free-for-all and teams, last one standing wins the round, first to N
  round wins takes the match. Target 4 players locally; more over the network.
  **No CPU bots** (David, 2026-10-04: baddies with real behaviours instead of
  generic bots). Battle is for people.
- **Revenge**: knocked-out players ride the arena edge and lob range-2 bombs in. An
  option (SB5's idea) lets a revenge kill bring you back in. Riders leave when
  sudden death starts.
- **Sudden death**: at the time limit, blocks fill in from the edge in a spiral
  (Mr.Boom's fill-order table is the model) or a per-arena pattern.
- **Battle stages**: a classic arena plus one per adventure world, each built
  around that world's gimmick.

## Adventure (approved 2026-10-04)

Built from the SB3, SB4 and SB5 research in `research/` (mechanics only).

**Premise idea** (to be decided): fireworks, not war. The hero makes fireworks.
The night before the Festival of Lights every firework in the workshop vanished
and came down across the worlds as **duds: unexploded ordnance**, the game's
namesake, turning the locals strange. The hero goes after them. Keeps bombs
festive and the tone playful, as the Name section asks.

**Worlds** (own themes; about 8 stages plus a boss each, so 70+ stages):

1. **Orchard**: gentle meadow, no gimmicks at first, then hedges that hide things.
   The plain walkers and the first mimic.
2. **Clockworks**: gears, conveyors, sliding blocks, a generator that winds up toy
   soldiers. Deterministic right-hand-rule walkers and hoppers that never path
   around obstacles.
3. **Kelp Deep**: currents that carry you, kelp that sways across lanes, burrowers,
   spear throwers, jellies that shock their 8 neighbours.
4. **Glacier**: ice you slide on, cracking floor (second pass makes a hole), a
   snowball that rolls down its column when freed, blast-jumpers, a splitter.
5. **Dune Ruins**: quicksand conveyors, sinkhole warps, curl-up immunity, a fire
   breather that backs off when you get close, webs that arc over walls.
6. **Forge**: eruptions onto shadow-marked tiles, a minecart track with a switch,
   bomb eaters, a translucent row-pacer, boulder mimics.
7. **Carnival**: springs, revolving doors, harmless bouncers, wall magnets,
   balloons that split, conga chains.
8. **Terminal worlds**, only possible here and uXo's own: **Scrollback** (the
   arena scrolls up like terminal output, new rows arrive at the bottom, the top
   row is lost) and **Static** (glitch baddies that warp, tiles that flicker between
   wall and floor, a cursor that types new blocks onto the grid).
9. The finale (story to decide).

**Map**: SB5-style branching. A cleared stage opens one to three exits; routes
decide which boss you meet and which ending you get. Start simpler (mostly linear
with optional side branches) and widen once the content exists.

**Stage objectives**, for variety: *Clear* (beat every baddie); *Duds* (set off
every dud, some carried around by walkers, after SB3's cores; you can skip
baddies); *Spawner* (destroy the generator structures that keep refilling the
stage); *Boss*. A timer per stage (about 3 minutes), lives, and handling items lost
on death, as in SB.

**Introduction curve** (all three games agree): a new species appears alone first,
then paired with the previous one, then in mixes. Each world brings 5 or 6 new
species and retires most of the old ones.

## Built: World 1, the Orchard (2026-10-04)

`adventure/orchard/*.txt` (stage format in `src/stage.rs`; laid out once by a
generator script, the files are the source of truth). Nine stages, each its own map:
Windfall Lane (classic, grubs), Beetle Rows (tree rows), Hedge Maze (a maze, hiding
hedgehogs, duds), Frog Pond (a rippling pond, frogs: the first eggs), Cider Press
(belts, duds), Ram Pasture (fences, rams: the dash mount), Crow Tree (a great canopy,
crows over the hedges), The Old Grove (everything, the last duds), The Bramblehog
(the boss: a 2x2 hedgehog that curls up, rolls at you flattening hedges, and calls up
grubs). Theme `orchard`: hedges with apples for soft blocks, flowers in the grass;
`w` water and `T` trees are solid tiles drawn as a pond and a canopy.

Rules (`src/adventure.rs`): 3 lives each; a lost life costs the handling items and
the mount, power-ups otherwise carry on stage to stage; a stage clock (time up costs a
life and restarts the stage); score (baddie points, 500 a dud, 10 a second left); the
exit hatch opens when the goal is met; firework celebrations; Continue from the
furthest stage (`$XDG_DATA_HOME/uxo/progress`).

## Baddies (approved; engine built in `src/baddie.rs`)

A whole roster, not bots: each species is **data**, composed from a small
vocabulary, so dozens of species cost little code. The research shows SB's
enemies are all built this way: a way of moving, plus a trigger, plus a reaction.

- **Moves**: wander (random turns), straight (turn only at walls), right-hand rule,
  chase when lined up, chase when near, hop N tiles, fly over soft blocks, burrow.
- **Triggers**: sees you in line (walls block sight), you are within N, a bomb is
  within N or in line, a timer, being hurt, an ally dying.
- **Actions**: charge until blocked; shoot (stopped by bombs, or not); flame cross;
  shock ring; self-destruct; eat, kick, punch or throw a bomb; set a bomb off from
  afar; turn a bomb into an item; spit or grow soft blocks; burrow or curl up
  (immune for a while); turn translucent; warp; split; pull you in; reverse your
  controls; stun; slow. Nothing that darkens or hides the screen (see Rendering).
- **Properties**: HP (1 to 4), speed, points, passes soft blocks, flies, disguised as
  a block until woken, segments (a snake whose body breaks off), contact harmless,
  drops an egg.
- **Structures**: spawners with HP that refill the stage until destroyed.

One text file per species in `baddies/` (format in `src/baddie.rs`), compiled in, so
new ones are written, not coded, art included (8x8 pixel rows, 16x16 for a size-2
boss, a second walk frame). Target 40 to 50 species across the worlds. Each gets its
own name and look; none reuse SB's.

Built so far (World 1): Grub (slow wanderer), Ladybeetle (straight on, turns only at
walls), Hedgehog (disguised as a hedge until you come close, then chases on sight),
Hopper (hops two tiles over anything; egg: hop), Ram (charges along a line it sees
you on; 2 HP; egg: dash), Crow (flies over hedges, chases on sight), Bramblehog
(boss). Implemented vocabulary: moves wander / straight / chase, `sees` + `on_sight`
chase / charge, abilities hop / roll on a timer, `summon`, `mimic`, `flies`, `size` 2.

## Mounts: tamed baddies (approved; built for hop and dash)

SB4's best idea, made central: **some baddies drop an egg when beaten, and the egg
hatches into a rideable version of that species with its signature ability**. A
hopper lets you hop over blocks, a charger dashes, a kicker kicks, a block-spitter
pushes blocks. The mount takes a hit for you. Up to two more eggs trail behind you
(blasts can destroy them, rivals can steal them). Every mount is a baddie you have
won over, so the roster and the mounts are one design, and eggs also appear in
battle.

## Our hero (to design)

Keep the family David liked: a round body, quiet single-pixel eyes, per-player
colour. Give it one memorable detail of its own (never a helmet with an antenna;
that is Bomberman's). Iterate with snapshots before settling.

## Build order (approved by David 2026-10-04: "LOVE IT! Go for it!")

1. **Battle complete**: rounds and match wins, revenge, sudden death; the full item
   set (kick, punch, glove, line bomb, remote, pierce, bouncing, mine, power bomb,
   heart, vest, passes) and skull diseases; battle stage files with the first
   gimmicks. *Done 2026-10-04 (power bomb was folded into full fire), plus a
   setup screen and the score section David asked for.*
2. **Adventure foundations**: the baddie engine and file format, stage files with
   objectives, lives and timer, eggs and tamed mounts, World 1 (Orchard) with its
   species and boss, and the hero art pass. *Done 2026-10-04 except the hero art
   pass.*
3. Worlds 2 to 4 and the branching map.
4. Worlds 5 to 8 (the terminal worlds), the finale and the story.

## Rendering (prototype built, `src/render.rs`)

**The arena is bigger than the view** (David, 2026-10-04): the battle maps are 25x17
and a camera follows the action. Auto zoom (the default) picks the largest size that
shows everyone still standing with room round them (and never closer than about a
classic arena's worth of tiles), zooms out at once when players spread and back in
only with room to spare, so it doesn't flicker; it pans smoothly and jumps on a new
round. `+`/`-` fix a size by hand, `0` returns to auto. An arena smaller than the view
is centred. Glyph mode scrolls by whole tiles.

**No darkness, no fog of war** (David, 2026-10-04: "it is too tight", "no fog of war").
The Blackout stage and its light radius were removed; don't bring them back.

**Screen layout**: a three-row top bar in a 3x5 pixel font (the uXo logo with a
flashing X, the stage centred and shortened if space is tight, the round and clock or
HURRY! at the right); a compact player row under it, or player cards at the sides when
the terminal is at least 190 wide; the play view; the hint row centred one row up from
the bottom, the last row left clear.

Terminal cells are roughly 1 wide by 2 tall, so the pixel sizes draw half-blocks
(`▀`, fg = top pixel, bg = bottom): two square pixels per cell, the pipeline Marble
Recovery proved. A tile is N pixels square; the largest size that fits the terminal
is picked, `+`/`-` overrides:

- **Hand-drawn art at 4, 6 and 8 pixels** per tile (N x N/2 cells per tile; the
  8 art is the most detailed).
- **Whole-number upscales above that**: 12 and 18 (the 6 art at 2x and 3x), 16 and
  24 (the 8 art at 2x and 3x), so art pixels never come out uneven. Flames are
  worked out per art pixel too, so they stay as chunky as the sprites.
- **Glyphs**: one glyph pair per tile (2x1 cells: `@1`, `()`, `B+`), arena 30x13,
  players snapped to tiles. A fallback minimap, not the way to play.

Arena size in cells is 15N x 6.5N: 60x26 at 4, 90x39 at 6, 120x52 at 8, 180x78 at
12. David's full-screen terminal (about 320x75) gets 8; added 2026-10-04 after the
first prototype topped out at 6 and left most of his screen empty.

Players are drawn at pixel positions, not tiles, so movement is smooth at every pixel size.
Floor tiles under a block get a one-art-pixel shadow row. Flames are computed per art pixel
(hot core plus half-arms toward linked neighbours, thinning as they burn out);
soft blocks glow and crumble pixel by pixel; bombs throb red faster as the fuse
runs down; the arena shakes a cell sideways on every blast. Drawing costs about
0.12 ms per frame at 8 pixels, 0.23 ms at 12.

Classic arena is 15x13 tiles including the wall (still to verify against the SB
games). The hero, bombs and item icons are placeholder art.

## Input (`src/input.rs`): gamepads are first-class

David's rule (2026-10-04): gamepads are first-class citizens here, not an add-on.

- **Every pad is its own player.** Any button joins (the join press is never also
  a bomb or a pause). Up to four seats; pads hot-plug, and an unplugged pad's seat
  waits for it ("pad unplugged" in the HUD).
- Left stick (deadzone 0.3) and d-pad. The dominant stick axis is where you go; the
  other axis, past 0.35, is the turn taken when the way ahead is blocked, so
  leaning into a junction turns the corner.
- A or X drops a bomb (both thumb buttons); B is kept free for kick/punch; Start
  pauses.
- **Rumble** via gilrs force feedback: blasts within four tiles (stronger closer),
  a hard buzz when caught, a tick on pickups and on joining.
- Only SDL-mapped devices count as pads (gilrs also reports drawing-tablet key pads
  and 3D mice; David's 3Dconnexion receiver is correctly ignored). `UXO_ANY_PAD=1`
  accepts everything.
- Keyboard: WASD + Space and arrows + Enter. With the kitty keyboard protocol both
  can play at once; legacy terminals only send autorepeat, so one key at a time
  (the HUD says so). Pads are the real answer to four players on one machine.
- Known gap: a pad that comes back under a different gilrs id takes a new seat.

## Simulation, AI and network

- Fixed-step deterministic simulation (integer or fixed-point positions, seeded
  RNG for item drops). That makes replays, rollback and iroh netplay cheap, as in
  Marble Recovery and Tetrix.
- **Baddie brains** (see Baddies): composed behaviours, not general bots. Smarter
  species (and bosses) can still borrow the proven pieces: a danger map with time
  until each cell burns (Bombermaaan `CAiArena`, Bombic 2 "flame presumption") so a
  baddie can dodge, and Bombic 2's graded difficulty ladder as a model for how
  cleverness grows across the worlds.
- Networking: iroh LAN discovery and WAN dial, embedded host, like Tetrix.
- Session logging per the Asciiverse convention (`log` + `env_logger`): raw input
  events, state transitions, periodic state snapshot.

## Levels

Every arena and adventure stage is an original design; no layout is copied from SB
or the GPL references. David asked for **real maps** (2026-10-04): each stage its own
place with its own shape, not the pillar grid with a gimmick sprinkled on.

Battle stages are `stages/*.txt` (format documented in `src/stage.rs`): `name:`,
`about:`, optional `give:` (items everyone starts with), `items:`, then the grid, any
size, walled all round (the six are 25x17): `#` wall, `+` soft, `?` soft three times in four, `.` floor, `1`-`4`
starts, `> < ^ v` belts, `R L U D` arrows, `o` warps, `~` ice. `cargo test` checks
every map parses, every tile is reachable and no start is boxed in.

1. **Open Field**: the classic arena.
2. **Conveyor Works**: factory floor, a belt loop round a machine room (belts carry
   players and bombs).
3. **Ricochet**: a walled court ringed by a racetrack; arrows send kicked bombs round
   it; everyone starts with kick.
4. **Warp Garden**: four walled quarters linked by warps (each sends you to the next).
5. **Ice Rink**: an oval rink behind the boards; on ice you slide until stopped.
6. **Catacombs**: a maze of crypt corridors with loops knocked through and three
   burial rooms.

The six 25x17 maps were laid out with a one-off generator script (base pattern plus
each map's structures); the `.txt` files are the source of truth now, edit them
directly.

## References

`references/bomberman/README.md` lists the 14 kept archives, their licenses and
what to read each for. Only MIT code (Mr.Boom AI, CBomber) may be lifted; the rest
are ideas only.

## Open questions / next steps

- [x] Name: uXo
- [x] Rendering prototype: half-block tiles 4 to 24 pixels, glyph fallback
- [x] Default size: auto-fit to the largest that fits (8 on David's screen)
- [ ] Movement and corner assist feel: tune by playing (`ASSIST_MIN`, speeds in `src/game.rs`)
- [x] Battle rules: rounds to three wins, last one standing; practice respawns
- [ ] Our hero and mount creatures: look and abilities
- [ ] Power-up naming and icon set
- [ ] Story premise (fireworks idea in Adventure)
- [x] David approved the Adventure, Baddies and Mounts proposals and the build order
- [x] Battle stage files and gimmicks: belts, arrows, warps, ice (darkness was
      tried and dropped)
- [x] Battle setup screen
- [ ] More battle maps and gimmicks: springs, cracking floor, rolling snowball,
      revolving doors, wall magnets, minecart
