# Super Bomberman 4 (Hudson / Produce!, Super Famicom, JP 1996): research notes

Gathered 2026-10-04 for uXo design. Reference only: mechanics as inspiration, never
names, art, layouts or music. Main source: the Bomberman Fandom wiki via its
MediaWiki API (`https://bomberman.fandom.com/api.php?action=parse&page=<Title>&prop=wikitext`),
whose enemy/boss/trap pages cite the *SB4 Hudson Official Guidebook* and the manual;
cross-checked with Wikipedia. "Unverified" marks gaps.

## Story mode structure

- 5 eras x 8 areas: 1 to 6 normal, 7 a rival bomber duel, 8 a big boss.
- Clear: kill every enemy, the exit opens. 2P co-op. Per-player item sets per area.
- Timer 2:59 (1-1, 1-2), 3:59 otherwise; none for rival/boss. 4-digit passwords per area.
- Caged allies in some areas (story flavour; gameplay effect unverified). Linear, no hub map.

| Era | Setting | Areas 1 to 6 (enemies, gimmick) | Rival / Boss |
|---|---|---|---|
| 1 Primeval | mountain, then cave | single species per area early; falling rocks, bridge, enemy safehouse | hammer rival / bomb-eating dino |
| 2 Edo | pagoda | three ninja types (with trolley), then pairs; trick floor | jet rival / sweeping mech |
| 3 Modern | skyscraper city | arrows, alarm device | lady rival / floating rocketeer |
| 4 Super Future | robotic jungle | signal floor, tunnel foliage | bazooka rival / centipede |
| 5 Hyperspace | coliseum | boss rush: four rivals, the great rival, final boss | |

Bonus stages A to I: 2P only, entrances behind hard blocks (throw your partner in) or
inside a block both players push at once. Egg room, slot machine, score blocks, warp
maze, fenced items.

## Enemy behaviours (the important part)

Default: walk straight; at a block/bomb/enemy pause, pick a random direction; touch hurts.
HP 1 to 2. Areas hold 3 to 5 enemies.

- Default walker (x2 species).
- Fast walker that sometimes stops and bounces in place.
- Slow walker that turns toward you when lined up; respawns from a safehouse fort
  (3x3, 8 blasts to destroy); sometimes rides a carrier.
- Carrier: only appears carrying another enemy; turns without pausing; kicks bombs.
  Killing it drops its rider.
- Charger that periodically burrows: invincible, harmless, passes soft blocks.
- Self-destructor: stops, becomes a bomb, blows at max range. Killed by its own
  kind's blast, scores nothing.
- Dodger: approaches in line of sight; waits and jumps over blasts in its line;
  remote bombs beat it.
- Thrower: in clear line, stops and throws one straight projectile.
- Death rocket: when hit it flies off, a marker shows its landing, explodes radius 6.
- Bomb puncher: walks to bombs in sight and punches them.
- Dasher: fast dash, slows, stops, dashes again (2 HP).
- Feinter: turns before reaching obstacles; when hurt flips and chases (2 HP).
- Shy one: turns edge-on (hard to see), moves away from you.
- Bomb sitter: rushes bombs in line and sets them off by sitting on them, unharmed.
- Kamikaze missile: launches at you in line, explodes on contact (fire 9).
- Tank: sticky shot that slows you (2 HP).
- Gas maker: slow; one homing gas cloud at a time; harmless, but a blast makes the
  cloud explode at max range.
- Grower: never moves; grows soft blocks on free neighbours every few seconds
  (growing ones are indestructible); sleeps once walled in.
- Bomb smith: rushes bombs in line and hammers them into a random item for you.
- Warper: chases in line; periodically warps, invulnerable meanwhile.
- Splitter balloons: when one dies, a survivor flickers (hittable) then splits to
  replace it.
- Igniter: after its first hit it catches fire, gets fast and pursues (2 HP).
- Telekinetic: detonates bombs in line, invincible while doing it, then powerless.
- Psychic: fast; a wave that reverses your controls, blocked by bombs (2 HP).
- Dancer: note wave range 5; stuns players into dancing, freezes enemies (2 HP).

Design lesson: behaviour = default walk + line-of-sight trigger + a reaction
(charge, shoot, flee, or a bomb interaction: kick, punch, detonate, dodge, convert).
New species introduced one at a time, then paired with the previous one.

## Ride the enemy (SB4's signature)

Rideable enemies drop an egg when killed: capture by defeating. Touch an egg to ride;
a hit kills the mount instead of you; stock up to 2 more eggs of the same family
(bio or mech), trailing behind you (blasts destroy them, touch steals them).
Abilities granted: pierce bomb, wall pass, dash, clear all soft blocks, bomb pass,
full fire, kick, punch, max speed, line bomb, slowing shot, kamikaze missile (mount
lost, rider briefly invulnerable), stun wave (works on rival bombers too).

## Bosses

Rivals: CPU bombers with a special move (chain sweep that stuns and scatters items;
invincible jet dash leaving fire; stun bits; forward beam; brief invincible stand),
some paying a 10 s weakness after using it. Big bosses, 10 HP class: a slow bomb
eater whose hits spawn bomb-hunting fireballs; a mech sweeping the top row firing
sliding bombs that can hurt itself; a floater only hittable after it slams down,
calling crosshair missiles and gas; a centipede with breakable segments, slowing
thread, then segments orbiting to swat bombs; a two-phase final boss after a boss
rush, with creeping tile-by-tile explosions.

## Stage gimmicks

Falling rocks (become soft blocks, stun, drop items); safehouse spawner; bridge (no
bombs, stops blasts, enemies cross); trolley on a random looping track (immune
rider, kills and breaks blocks); trick/spring floor (throws you to a linked tile);
arrows (redirect kicked bombs); alarm block (breaking it enrages every enemy);
signal floor cycling solid / walkable-no-bombs / launcher; foliage tunnels hiding
what is under them.

## Items

Fire, bomb up, speed up, remote, bomb pass, invincible suit, speed down, egg, pierce,
block pass, kick, punch, **push** (lunge one tile, shove an opponent 3 tiles), full
fire, power glove (can throw players too), heart (one hit), 1up, score food, time,
select item, costume (battle). Skull diseases (battle; pass by touch or cure with any
item): max speed, slow, constant bomb drops, no bombs, min range, can't stop,
reverse, short fuse, long fuse, invisible, drops items, swaps places with a player.

## Battle

Up to 5 players. Royale (single or tag teams), a champion ladder against the rivals,
maniac mode (item counts and frequencies, 1 to 5 HP). Options: sudden death
(pressure blocks), revenge riders (fire-2 bombs from the border, removed at sudden
death), winner minigame for a next-round item. 12 stages, each with 4 layout
variants: classic; time flow switching slow/normal/fast; iron balls blasts kick
around; cliffs you stagger off; electric rope border; floors re-rolled by blasts
(normal/slide/slow/skull); foliage + arrows + vertical wrap; slot machine with no
soft blocks; seesaws launching bombs or players 6 tiles; costumes + signal floors;
one trolley; flag tag (lose your flag, you die).
