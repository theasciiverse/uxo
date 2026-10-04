# Super Bomberman 5 (Hudson, Super Famicom, 1997): research notes

Gathered 2026-10-04 for uXo design. Reference only: mechanics as inspiration, never
names, art, layouts or music. Sources: Bomberman Fandom wiki via its API (zone pages
cite the SB5 Hudson Official Guidebook), Solomon warrior's GameFAQs FAQ v3.2 (2003,
video-games-museum.com PDF mirror, includes the stage graph), Wikipedia.

## Story mode structure

- Five zones. Zones 1 to 4 restage the four earlier games (enemies, traps, scenery);
  zone 5 is all new (railroad, toy box, fun fair, electromagnet, clockwork), 30 stages.
  Zones 1 to 4: 15 stages each plus 2 to 3 boss rooms.
- Clear: kill every enemy; **several exits appear, you choose one**. Exits lead to new
  or already cleared stages (sources disagree on the exit colours).
- **Branching map**: 1 to 5 exits per stage, many linking backward or sideways, so zones
  are loops, not trees. Your route decides which boss you meet and which entry stage
  of the next zone you start from. Good ending via one route into the final room, bad
  ending (villain escapes) via the others.
- Saves a completion percentage; 100% unlocks a gold battle character and an "encore"
  replay with items moved (toward 200%). 2P co-op; some items only appear with 2P.

## Enemy behaviours (zone 5 is the new material)

Zone 1 to 4 restaged enemies include: random walker; slow one that sits, becomes a
bomb and explodes (invulnerable sitting); flame shooter (4 tiles, sets off bombs, 3
HP); **invisible-at-times** wanderer (only eyes show, invulnerable then); **bomb
chaser that eats bombs at any distance** (two bombs on opposite sides make it
flip-flop); bomb-and-wall-only turner that ignores you; armoured fast charger (4 HP);
soft-block passer; **magnet** that pulls you in within 3 tiles; shocker hitting one
tile in each direction when lined up within 4; one that is stunned long by its first
hit then speeds up; non-igniting flame; **belly-slider** charging when lined up within
6; **sleeper** vulnerable only while snoring; plus the SB3/SB4 sets.

Zone 5:
- Chaser that stuns with electricity, invulnerable meanwhile; groups of 4 **respawned by
  a tower** structure (4 pillars, 4 hits each, stun waves every ~10 s).
- **Hoppers that hop straight at you and never path around**, so they get stuck.
- **Right-hand-rule walkers** (forward, turn right at an obstacle, left only if right is
  blocked, reverse only at a dead end) that chase when aligned; groups respawned by a
  **generator** (4 HP). Deterministic and readable: suits a grid/terminal game.
- Slow chaser that fires springs; flamethrower (4 tiles, doesn't ignite bombs).
- **Harmless bouncer**: bumping it bounces you 6 tiles (from a generator).
- Bat that chases within 3 tiles (diagonals too) and **shields with its wings** when a
  bomb lands in front.
- Bomb converter (aligned bombs become one of 8 items).
- **Snake**: head plus 4 segments; hitting a segment breaks it off (head speeds up,
  segments jitter); killing the head first kills the whole body; one variant's head
  always drops an egg.
- **Soft-block mimic that even blocks wall-pass**; revealed by a blast; drops an egg.
- Shoveller that drives to aligned bombs and **throws them 6 tiles back**.
- Stretcher (extends 3 tiles); **tongue** that eats aligned bombs from 3 tiles.
- Slow and fast diagonal-aware chasers (2 HP).

Lesson: late threats shift to **spawner structures** you must destroy, disguised
blocks, **bomb manipulators** (eat, steal, convert, throw your bombs), and
**multi-segment** enemies.

## Bosses

Bomber-type bosses collect power-ups and don't hurt on contact (extra HP in 2P): a very
fast one burying invisible mines and stealing power-ups; a dodging pair (one shoots
along your row, one slashes on your tile); a slow block-maker; one with homing bombs; a
rider throwing speed-down items in an arena ringed by pits; one that keeps 3 minions
summoned; one that spawns skulls and speed-downs on a block-less arena; one whose touch
scatters your items. Final, three forms: bell cannons dropping L-shaped bomb chains
(vulnerable only while open); a roller that tracks you then rolls down the screen,
knocking bombs away (vulnerable while its arms are grounded); the villain on foot,
fastest unit in the game, throwing bombs two at a time behind a spinning blast-proof
barrier, then tossing out a clock that freezes the other side for whoever grabs it.

## Stage gimmicks (new in SB5)

All soft blocks blink invisible together; dynamite hard block (plus-shaped burst);
**booster tile** (a bomb on it gets range 10); furnace pulling in a bomb, then fire
running along an oil trail every ~10 s; regenerating soft blocks; trampolines;
rotating arrows that turn when blasted; **pipe crosses that multiply a blast to the
arena edges**; sticky floor (slows players only); **wall magnets** (5 s off, 8 s on,
pulling every bomb); hard block sliding along a track; gear-driven moving floor
(blasting the gear stops it 7 s); wall saucer launchers (blink, then fire); spike blocks
that knock an item out of you; **bomb warp** (teleports a bomb randomly); fall zones
around boss arenas (stagger at the edge, then fall).

## Items

Bomb up, fire up, full fire, speed up, kick, power glove, boxing glove (punch), bomb
pass, wall pass; special bomb types where only your first bomb is special: remote,
pierce, **mine** (buried, triggered by stepping), **search bomb** (homes on targets
within 3 tiles, blows after ~3 s); heart, vest (temporary invincibility), 1up, clock
(freezes enemies), speed down, egg. Skull list for SB5 unverified (at least: can't
stop, reverse, short fuse, long fuse).

## Mounts

Kick soft blocks; jump over blocks, bombs, enemies, traps and blasts; kick bombs; line
of bombs; high-speed roll; punch players to stun them. A mount dies in your place.
No new egg spawns if 2 are already on screen.

## Battle

5 slots (human or CPU, CPU levels), single or tag. Options: wins 1 to 5, time 1:00 to
5:00 or unlimited, sudden death random/on/off, **revenge with a "super" setting where a
kill from the cart brings you back** (brief invincibility, base stats), winner bowling
minigame. Maniac mode: pick up to 60 items from 15 types, handicaps. Custom character
builder. 13 stages x 4 layout groups: classic; conveyors; roof cover hiding things;
**two spotlights in darkness**; full throttle (everyone very fast); snow and ice; arrows;
cart with a switch; magnets and sliding blocks; **no soft blocks and max fire**; 18
random warps; conveyors with bomb-only tunnels; a lethal cart through warps.
