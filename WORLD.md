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
| (World 2) | the Clock | tries to wind the system clock on to 4:00 so the job ends sooner: the reason the Clock is the second world |
| (World 3) | the Buffer | ... |
| 04:00 | purged's fight | the act's ending |

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

## Decisions

David, 2026-10-29: Dot is the hero and the player plays Dot; co-op's second
hero is Caret, with a pointed head; the countdown is clock time with the
wipe at 4:00 on the cards; nothing dies; bombs are kills and a UXO is a
hung process; text and visual touches both go into the release.
