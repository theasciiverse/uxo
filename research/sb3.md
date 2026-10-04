# Super Bomberman 3 (Hudson, SNES, 1995): research notes

Gathered 2026-10-04 for uXo design. Reference only: mechanics as inspiration, never
names, art, layouts or music. Sources: Bomberman Fandom wiki (pages citing the SB3
Hudson Official Guidebook and manuals), Ragey's Bomberman Shrine
(randomhoohaas.flyingomelette.com/bomb/snes-3/, via Wayback), Hardcore Gaming 101,
Wikipedia. Disagreements between sources are noted.

## Story mode structure

- 1 or 2 players co-op, friendly fire on. Six worlds in order: jungle/swamp, volcanic
  mines, seabed, desert pyramid, snow, the villain's battleship (7 areas).
- Areas of 1 to 4 single screens (no scrolling). 3:00 per stage; timeout costs a life.
- **Clear by destroying all cores** (2 to 4 per screen), not by killing every enemy.
  Some cores ride slow **walkers** (kill the walker, then bomb the core). Then the exit
  opens; some exits are a capsule holding a story piece. Enemies can be skipped.
- Stage-select hub; cleared stages replay with no cores (item farming).
- World 1 is "largely gimmick-free"; each world adds its own gimmicks.

## Enemy behaviours by world (6 new types per world, the old set retired)

W1 jungle: chaser that only turns at junctions (sometimes lingers instead); straight
walker turning only at walls; **tree mimic** that looks like a soft block until you
come close (killable while disguised); **hopper** jumping 2 tiles over blocks and
bombs, immune in the air; fast slime turning only at walls; flier over soft blocks.

W2 volcano: **boulder mimic** that wakes on approach or blast, wanders, then turns back
into a block (only hurtable while awake); fast erratic fire-wheel; flier; slow
territorial one that periodically makes a range-2 flame cross (sets off bombs);
**row-pacer** that tracks your row or column and periodically turns translucent and
blast-immune (a trolley still kills it); **bomb eater**, a slow chaser.

W3 seabed: **burrower under sand** (immune) that surfaces to **spit ink that darkens
the screen** (blast it mid-attack to cancel); diving chaser (dives when bombs are
near); homebody that chases when you come close and **shocks all 8 neighbours**;
**spear thrower** on line of sight (2 HP; bombs block spears); fast one that extends
tentacles 1 tile when adjacent; slow **leaper** (jumps 2 tiles, clearing bombs and
blocks under it, immune mid-air).

W4 desert: slow chaser that **burrows when a bomb is within 2 tiles**; chaser that
**retreats ~4 s when within 2 tiles of you** and breathes fire 2 tiles; slow one
firing a web that **arcs over obstacles** to land on the 5th tile; fast spore shooter
(spore stops at bombs/walls); chaser that **curls up immune** near you or bombs, then
cools down; slow circling whirlwind that **sucks you in** within 2 to 3 tiles; tougher
2 HP core walker.

W5 snow: walker that **jumps over a blast** it meets if there is room (2 HP);
relentless slow chaser with a **ball and chain** (3 tiles, stopped by bombs/blocks);
**curl-and-jump** mover (left/right only, 2 tiles, immune mid-air, lands on a bomb you
drop there); **charger** when you enter its line within 4 tiles, until bomb or wall;
fast **block-spitter** that shields itself with a soft block when it sees a bomb (2
HP); burrower that surfaces to spit a cold star on alignment and **splits into 4 fast
little ones** on death.

W6 battleship: **robot mimic** that wakes and smashes soft blocks on both sides (and
bombs and players there), never sleeps again (2 HP); territorial spinner spraying fire
all ways, jumps away from blasts (2 to 3 HP); gunner whose bullet **goes through
anything**, breaking blocks and setting off bombs, periodically flying (2 HP);
**conga chain** robots that link up to 8, scatter diagonally when blasted, only
hurtable when single; hopping chaser that boosts when aligned; **self-destructor** that
periodically explodes at max range through soft blocks.

Design verbs: soft-block mimicry, temporary immunity (burrow, curl, jump, translucent),
reacting to bombs (eat, shield, flee, dodge), line-of-sight trigger (shoot, charge,
boost), telegraphed random-tile strikes (shadow marks).

## Bosses (6 to 8 hits)

Fox spirit whose leaves turn bombs into fruit, mounts into turtles and players into a
bomb-less rabbit for 7 s, and who becomes a dashing wisp with orbiting minis; rock mech
with fire-whip arms and **wall-punch screen shakes that paralyse players while bombs
keep ticking**, fire stream and shadow-marked volcano bombs; turtle mech hittable only
at the head while still, slowing steam, water streams that carry players and bombs;
**twin cacti** with separate HP passing a boomerang hat, spines, and a guitar that
speeds the other up; a floating mecha princess with a chasing tornado, snowflake
spread and bomb-kicking minions; the villain with a shield and max-range rubber bombs
who spawns self-destructors; a merged final mech with telegraphed attack icons,
rolling bombs that blow at every intersection, homing missiles and sweeping lasers.

## Stage gimmicks

Foliage hiding what is under it (in battle a bomb under it blows at max range); bridge
(no bombs, blasts stop, monsters can't enter); volcano raining shadow-marked fireballs
(break blocks, set off bombs, hurt players only); **trolley** on a track with a
switchable junction (immune rider, destroys everything in the way); ocean current
(shoots you along, pushes you back 3 against it); quicksand conveyors with a
direction-switch panel; pyramid with a snake popping from one of 4 holes after an eyes
warning; pit warps; **cracking floor** (first pass cracks, second makes a hole; you can
trap yourself); igloo hiding its inside; **giant snowball** resting on soft blocks that
rolls down its column when freed, killing everything; electric antenna pairs that
flash then zap between; missile blocks that launch to a random marked tile when bombed;
exploding boxes that blast to the nearest walls. Battle extras: revolving doors,
crushing 3x3 block that lurches after flashing its eyes, spring fountain (4 tiles),
fixed and rotating arrows for kicked bombs, seesaw catapult (6 tiles), arrow blocks
that move one tile when blasted from the far side.

## Items

Bomb up (max 8), fire up (max 6), full fire, speed up (max 9), speed down, kick (button
stops a kicked bomb), glove (throw 3 tiles; landing erases an item), rubber bomb
(kicked bombs bounce back; thrown ones bounce), power bomb (first bomb max range),
pierce, remote, bomb pass, block pass; story only: clock (freezes enemies and timer
16 s), invincible suit (16 s), heart (one hit), 1up, score fruit; egg. Items marked
lost on death: the handling ones. Skull (battle): fast, ultra-slow, nonstop bombs, no
bombs, min blast, can't stop, reverse, short fuse, long fuse, swap places with a random
opponent; passed by touch or ejected by picking up any item.

## Mounts

Egg hatches on touch; the mount is one extra hit (then ~1 s invincibility). Five
abilities: uncontrollable dash until impact; kick a bomb 3 tiles through the air over
blocks; line bomb; jump in place or 2 tiles over a block or bomb (can hop a blast);
**kick a soft block** that slides and passes its push along a row of blocks.

## Battle

Up to 5 players (first in the series), CPU levels, single or tag teams. 12 stages:
classic; revolving doors; currents + springs; pit warps + conveyors; cracking floor +
igloo + snowball; arrow loops + vertical wrap; pits + quicksand + crushing block;
trolleys; seesaws (no eggs); foliage with max-fire bombs and no fire-ups, wrap;
exploding boxes + doors; arrow-block puzzle. Each stage lists its exact hidden items
(e.g. 81 soft blocks, 28 items). **Revenge carts debut**: eliminated players ride the
edge lobbing range-2 bombs, removed at sudden death. Sudden death: pressure blocks in a
clockwise spiral from the top left. Winner's prize minigame; gilded winner option.
Passwords give remixed stage layouts.
