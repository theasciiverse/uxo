# uXo: the world inside the machine

**Status: settled with David, 2026-10-29, and built for World 1.** The
rules of the world, then what act 1 and the Screensaver take from them.
New worlds, cards and creatures are checked against the rules here.

## Where the world is thin today

- **The Screensaver is the Orchard with purged written over it.** Only the
  first card says you are inside a computer. The stages, creatures, art and
  every later card are a garden; nothing in them could only happen inside a
  machine.
- **Who Dot is, is unclear.** Card 1: "the screen pulled you in ... You with
  it, Dot!" reads as if you and Dot were two people.
- **The countdown is named but never felt.** "When the countdown ends": what
  countdown, ending when? The stage clocks are just stage clocks.
- **Why things happen is unexplained:** why the creatures turned on you
  ("gone strange"), why Dot has bombs, why blowing up a charge defuses it,
  what happens to a beaten baddie, what an egg is, why a bomb fizzles.
- **Leftover names:**
  - `src/main.rs:1135` says "the Clockworks are next"; the ending says
    "Next: the Clock!"
  - The stage-clear tally says "duds set off"; the goal line and the cards
    say charges.
  - `baddies/bramblehog.txt`: "the Orchard's old guardian";
    `baddies/grub.txt`: "a fat orchard grub".

## The rules of the machine

One page of rules, so every world, card and creature can be checked
against them.

1. **The machine** is one old home computer, left on overnight in a
   dark room. Never named, never a brand. Each world is one part of it.
2. **Dot is the hero, and the player plays Dot** (David, 2026-10-29). The
   blinking underscore under Dot's feet is a cursor; the way out at the
   very end, the Prompt, is a cursor too, so the game ends where it began.
   **In co-op the second hero is Caret** (proposed name, David's idea): on
   Windows the text cursor is officially called the caret, so Dot and Caret
   are both cursors. Caret has a pointed head, the `^` itself, and its tip
   blinks at the same terminal pace as Dot's underscore: Dot blinks below,
   Caret blinks on top (David, 2026-10-29: "give him a point head").
   Checked 2026-10-29 (two web searches, not a trademark search): no game
   or game character called Caret.
3. **Dot's bombs are kills** (David, 2026-10-29). Each bomb is a little
   `kill` Dot starts: the fuse is it counting down, the blast is the
   signal going out, and a chain is one kill setting off the next. purged's
   creatures have none; that is Dot's edge.
4. **A UXO is a hung process** (David, 2026-10-29). Now and then a kill
   hangs instead of firing, and lies there grey, stuck and still live,
   until a blast wakes it, it is knocked loose (kicked, punched, thrown),
   or it comes unstuck by itself. The Duds disease makes every kill you
   start hang.
5. **purged** is the cleaning daemon that runs every night at 3:14. Tonight
   it decided the tidiest disk is an empty one. A daemon can only delete
   things, it can't fight, so it works through what is already there: it
   rewrites the creatures of every program it passes to guard its charges.
6. **The countdown is real clock time.** purged's job ends at 4:00, and
   when it ends every charge left goes off at once and the disk is wiped.
   Act 1 is those 46 minutes. Each stage card shows the machine's clock
   (03:14, 03:17, ...), so the player watches four o'clock come closer.
7. **A charge set off early only wipes the tiles round it,** not the disk.
   That is why you blast them: controlled detonation, the way real UXO is
   cleared. It also explains why a charge goes off with a real blast.
8. **Nothing dies.** A kill ends purged's rewritten copy of a creature, and
   its program respawns the original, back in its own loop as it was
   before purged touched it (the poof is it going home).
   The tone stays cartoonish, as the Name section asks.
9. **An egg is a copy.** Some creatures leave a copy of themselves when
   they snap back, and the copy is on your side: that is your mount.
10. **The hatch** is the way from one program to the next. It opens when
    the program has nothing left of purged's in it.

## Act 1 over the clock

| Machine time | Where | What purged does |
|---|---|---|
| 03:14 | The Screensaver | starts its run; you fall in |
| 03:47 | the Screensaver cleared | notices its charges going off, one by one |
| 03:48 to 03:59 | the Clock | winds the system clock on toward 4:00 so the job ends sooner; the cards run fast |
| 03:52 | the Clock cleared | the Cuckoo stopped, the hands spring back to the true time |
| 03:53 to 03:58 | the Buffer | packs its last charges into the Buffer |
| 03:59 | purged's fight | comes itself, and is stopped |
| 04:00 | act 1's ending | four o'clock passes and nothing happens |

## The Screensaver, made a screensaver

It is an old garden screensaver, drawn in few colours, that has looped on
this machine every night for years. Its creatures are its sprites, each on
its loop: the beetles were drawn to walk the rows, the frogs to hop every
few seconds, the cider press to turn forever. purged rewrote those loops.
That matches the baddie rules already in the game (straight, hop on a
timer, chase on sight), so the mechanics become the story.

**The cards** are in `adventure/orchard/*.txt` (the stage files are the
source of truth): each carries the machine's clock, 03:14 at Windfall Lane
to 03:44 at the Bramblehog, and the ending stops the clock at 3:47,
thirteen minutes to four. Card 1 sets out the night; card 2 brings in the
hung kills, card 3 why a charge is blasted, card 4 a kill sending a
creature home and the egg as a copy.

Built 2026-10-29: the cards (`clock:` lines, `{dot}` for the heroes'
names in co-op), the ending, and the visual touches below.

**Visual touches (built 2026-10-29):**

- A beaten creature flickers out pale, as the screensaver drew it, rising
  a little before it is gone (`Homing` in `src/game.rs`, `HOMING` steps).
- Every 20 seconds a thin redraw line runs down the Screensaver, the way old
  screensavers repainted; it only lightens, never hides (`REDRAW_EVERY` in
  `src/render.rs`).
- The title line "Blast your way out to the real world!" ends in a blinking
  amber cursor.
- Caret's art (`caret_art` in `src/render.rs`): player 2, in battle too.

## The Clock

The program that keeps the machine's time. Everything in it moves on the
beat: shutters open on the tick or the tock, ticks walk the walls, bobs
swing, springs hop. purged's trick here is time itself: it winds the clock
on, so the cards gain a minute or two a stage (03:48 at the Second Hand,
03:59 at the Cuckoo), and its boss is the Cuckoo, wound to strike four
early. Beating it ends the winding: the hands spring back to the true time,
3:52, and purged turns to the Buffer. Cards in `adventure/clock/*.txt`.

## The Buffer

Where everything the machine is about to write waits in line. Its data
blocks are written back a few seconds after they are cleared, unless
something stands in their place. Its creatures are the Buffer's own (bytes,
packets, pointers, caches, overflow), rewritten by purged. At the end
purged comes itself: a daemon can't fight (rule 5), but it can delete, so
whatever it touches is gone. Drawn as a pink eraser. Beating it is a kill
like any other (rule 8): purged stops, and runs again tomorrow at 3:14 as
it was written, tidying and never wiping. Cards in `adventure/buffer/*.txt`.

## Act 2: the Forgotten Game

**Built 2026-11-14.** What the rules above become after four o'clock.

- **The Forgotten Game** is an old shareware bomb game, thirty years on the
  disk and never played. Its name is never given; it is "the old game".
  Its title has blinked PRESS ANY KEY every night all that time. At 3:14 a
  key was pressed (act 1's first card), and it felt it too. With purged
  stopped, it pulls Dot (and Caret) down: at last, players.
- **The stake is the clock again.** At five o'clock its demo starts over,
  and whoever is still inside plays it forever. Act 2 is that hour, 04:01
  to 05:00, on the cards.
- **Its three episodes are three worlds.** Episode 1 was the free one, and
  was archived into Cold Storage; episodes 2 and 3 were never paid for:
  episode 2 was deleted into the Recycle Bin, and episode 3 is at home in
  the Games Folder, beside every other game the machine ever played.
- **Its creatures are its own sprites**, not purged's rewrites, drawn in an
  old game's flat, bright colours. Rule 8 holds: a kill sends them home,
  and rule 9: some leave copies to ride.
- **Its bombs are its own** (rule 3 is about purged, whose creatures have no
  kills). In the Games Folder its creatures are bombers like Dot and lay the
  game's bombs, round and purple; they never hurt the game's own creatures.
- **The branching map is the game's own level map.** Each world shows it
  after every stage: two ways from the first stage to the boss, the upper
  mostly puzzles, the lower mostly action, crossing in the middle.
- **Its hero is Demo**, who has played the attract-mode demo every night
  for thirty years and never lost; as the last boss it walks the walls the
  same way every time, so its route can be learnt. Checked 2026-11-14:
  "Demo" is a plain word, fine for a character inside uXo, like Dot.
- **The ending:** the game's first GAME OVER in thirty years, Dot's name at
  the top of the high score table, and Demo happily playing its demo again
  at five, because now somebody has played. Under Dot's name a second new
  one nobody typed: PAYLOAD, act 3's villain, there all along.

| Machine time | Where | What happens |
|---|---|---|
| 04:01 | Cold Storage | the old game thaws out and starts its first episode |
| 04:17 | the Freezer | episode 1's boss |
| 04:19 to 04:35 | the Recycle Bin | episode 2, deleted, played anyway; the Compactor |
| 04:38 to 04:58 | the Games Folder | episode 3 at home; Demo |
| 05:00 | act 2's ending | the demo starts over, and Demo plays it happily |

### Cold Storage

Where files nobody opens are packed away and frozen: a frosted slate floor,
ice-cube blocks, ice lanes (`~`). Its own tile is the **packed archive**
(`=`): solid, and a blast stops at it and unpacks it, its tile clearing and
soft blocks spilling onto the free floor beside it. Species: Puff (wanders),
Skid (straight and fast), Packrat (hides as an archive; 2 HP), Hail (hops;
egg hop), Plow (charges; egg dash), and **the Freezer** (freezes the floor
round it into ice, charges, calls up puffs). Cards in `adventure/cold/`.

### The Recycle Bin

Where deleted files wait before they are gone for good: a dark mesh floor,
crumpled paper for blocks. Its rule is **restore**: a beaten creature (one
that leaves no egg) leaves its deleted file, and the bin restores it after
`RESTORE` (7 s) unless someone walks over the file first, emptying it for
good (100 points). A stage to clear needs its files emptied too. Species:
Scrap, Clip (straight, fast), Wad (hops; egg hop), Fade (flies, follows on
sight), Shortcut (charges; egg dash), and **the Compactor** (curls up and
rolls, flattening the heaps; calls up scraps). Cards in `adventure/bin/`.

### The Games Folder

The old game's home: its purple checker floor and floppy disks for blocks.
Its creatures lay the game's bombs (`ability: bomb`, `fire:`). Species:
Fizz (a wandering bomber), Stick (straight, fast), Bonus (hops; egg hop),
Diskette (hides as a floppy disk; 2 HP), Turbo (charges; egg dash), and
**Demo** (walks the walls, right-hand rule, laying fire-3 bombs). Warps
(`o`) join the Warp Zone's four rooms. Cards in `adventure/games/`.

## Decisions

David, 2026-10-29: Dot is the hero and the player plays Dot; co-op's second
hero is Caret, with a pointed head; the countdown is clock time with the
wipe at 4:00 on the cards; nothing dies; bombs are kills and a UXO is a
hung process; text and visual touches both go into the release.
