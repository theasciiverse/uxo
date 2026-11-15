# Roadmap

uXo is at 1.0.0 (2026-11-15), played through by a script and tuned; it
was at 0.9.0 (2026-11-14): battle mode complete and all three acts of
the adventure in, ten worlds and 90 stages: act 1 (the Screensaver, the
Clock, the Buffer and purged's fight), act 2 (Cold Storage, the Recycle
Bin, the Games Folder and Demo's fight, on the Forgotten Game's branching
maps) and act 3 (the Power Supply, Scrollback, Static, the Prompt and
Payload's last copy, and the way out), with the world's rules settled
(`WORLD.md`). David's own play-through by hand is still to come. The
adventure is a story in three acts, inside the machine, built one world
at a time.

## 0.3.0: built

- [x] Battle: rounds and matches, revenge riders, sudden death, six maps,
  the full item set and the skull's diseases, a setup screen, awards
- [x] Adventure foundations: baddies as data, eggs and tamed mounts, lives,
  score, clock, saved progress
- [x] The Orchard: nine stages, six species and the Bramblehog
- [x] Keyboard and every gamepad its own player, with rumble; session logging

## 0.4.0: built

- [x] The surge: a bomb whose blast floods outward round corners, for open
  spaces
- [x] Puzzle tiles: crates to push, keys and doors, switches and gates,
  parts and sockets
- [x] UXOs: bombs that fizzle and lie there live; the Duds disease; a
  battle setting for how often
- [x] purged's charges go off with a real blast
- [x] Names: Dot the hero, purged the villain, checked against games
- [x] Act 1's opening, the title line, and World 1's cards and ending
- [x] Dot's own art, with the blinking cursor underscore
- [x] World 1, the Screensaver: the Orchard rebuilt, stages that mix bomb
  puzzles and fast action, with crates, keys and doors, gates, parts and the
  surge; starting a stage over from the pause screen
- [x] World 1 played through and tuned
- [x] The store page and README with the story

## 0.5.0: built

- [x] The world's rules (`WORLD.md`): bombs are kills, a UXO is a hung
  process, purged's run from 3:14 to the wipe at 4:00, nothing dies
- [x] Caret, the second hero in co-op, with a pointed head
- [x] The Screensaver's cards rewritten, each with the machine's clock;
  the ending at 3:47
- [x] Beaten creatures flicker home; the Screensaver's redraw line; the
  title's blinking cursor
- [x] A new preview recording with Dot and Caret

## 0.6.1: built

- [x] Gamepads work on Windows: gilrs reads them through XInput, since its
  default (Windows Gaming Input) only reports to a program with a focused
  window of its own, which a terminal game never has

## 0.6.0: the rest of act 1, built

- [x] World 2, the Clock: nine stages, five species and the Cuckoo, shutters
  on the clock's beat, the cards running fast as purged winds the clock on
- [x] World 3, the Buffer: nine stages, five species, data blocks that are
  written back after you clear them
- [x] purged's fight at the end of act 1 (the Buffer's last stage), and its
  ending
- [x] Continue starts at any stage reached; a playtest log
- [x] Creatures catch you only on a real overlap
- [x] A new preview recording in the Clock

## 0.7.0: act 2, the Forgotten Game, built

- [x] Cold Storage: nine stages on the ice, packed archives that a blast
  unpacks into soft blocks, five species and the Freezer, which freezes the
  floor round it
- [x] The Recycle Bin: nine stages where a beaten creature's deleted file is
  restored unless you walk over it first, five species and the Compactor
- [x] The Games Folder: nine stages where the game's own creatures lay its
  bombs, five species and Demo, the game's hero, who walks the same way
  every time
- [x] The branching map: each act 2 world is the Forgotten Game's own map,
  two ways through (the upper mostly puzzles, the lower mostly action),
  crossing in the middle; six stages to the boss, nine to see everything
- [x] Act 2's ending, and Payload's name on the high score table
- [x] Progress is saved in the state folder (`$XDG_STATE_HOME/uxo`); 0.6's
  is still read

## 0.8.0: act 3, the Virus, built

- [x] The Power Supply: live wires with a pulse of current running along
  them; Spark, Fan, Cap, Arc, Jolt and the Big Fan
- [x] Scrollback: tall stages whose top row scrolls away every few seconds,
  and whoever is on it (charges go off as they go); Echo, Tail, Grep, Cat,
  Pipe and the Pager
- [x] Static: tiles that flicker solid and open, each on its own beat;
  Snow, Noise, Glitch (warps), Typo (types blocks), Burst, and Payload's
  first fight
- [x] Act 3 runs straight on, no map: a race to the modem at six

## 0.9.0: the Prompt and the way out, built

- [x] The Prompt: Payload's last copy types its dial-out command onto the
  command line (`,`) a character at a time; a blast deletes characters,
  and with the whole line typed it presses Enter (a life, and the stage
  again); the HUD shows how much is typed. Arg, Flag, Wildcard, Alias,
  Root, Bang, and Payload's last copy, who lays logic bombs
- [x] The way out: the game's ending, and THE END
- [x] Tuned by scripted checks: each new stage's start is safe for a few
  seconds, every hatch can be reached and no locked hatch can be walked
  round

## 1.0.0: played through and tuned (2026-11-15)

Tagged on David's go-ahead to tag 1.0.0 once a game is thoroughly played
and tuned. **David's own play-through by hand is still to come**; what
was played here was played by a script.

- [x] A playtest bot (`src/bot.rs`): it plays player one by trying each
  plan (walk somewhere, drop a bomb, walk out of the blast, push a crate
  onto a switch) on a copy of the deterministic game and only plays one
  that keeps Dot alive. `uxo --bot-test all 2` plays every stage cold,
  `--bot-test through SEED [FROM]` a whole run carrying power-ups, and
  `UXO_BOT=1 uxo` lets it play the real game in a terminal (tmux). It
  proves stages can be cleared and finds where it gets caught; it says
  little about how a stage feels to a person
- [x] Played through: whole runs from 1-1 and cold from 7-1 to THE END,
  every stage cold on two seeds, the last two stages and the ending in
  the real game in tmux. A run from 1-1 cleared all 81 stages of its
route to THE END with 9 game overs on the way (6 of them on the Hedge
Maze, whose clock went from 180 to 210 s); a cold run from 7-1 reached
THE END with none. Cold, two seeds each, it cleared 166 of 180 tries:
every stage at least once except the Zip Lines (on ice the bot steers
badly and runs out of time hunting Skids; it cleared it 3 times in 10
other tries, and its clock went from 170 to 200 s) and the Heat Sink
(cleared in both whole runs)
- [x] Tuned from it: the Zip Lines' start (a Skid slid into Dot's corner
  six seconds in); Scrollback waits 8 s before the first row goes (was 4),
  respawns a player well clear of the next rows to go, and drops keys and
  parts ahead of the scroll instead of eating them (The History could be
  lost for good, a soft-lock until the clock ran out; its key now starts
  lower too); the Heat Sink's upper fin has a Spark for a Jolt and 260 s
  (was 220); starting cold (Continue, or again after a game over) brings
  the act's power-ups (act 2: 2 bombs, fire 3, speed 1; act 3: 3, 4, 2),
  as a player arriving would have them
- [x] THE END stays up: the press that closed the last ending used to skip
  it straight to the title
- [ ] David's play-through by hand, and any tuning it asks for

## After 1.0

- [ ] More battle maps using the later worlds' tricks
