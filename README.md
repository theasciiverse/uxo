# uXo

**BLAST YOUR WAY OUT TO THE REAL WORLD!**

3:14 in the morning. One key pressed, and the machine pulled Dot in! Every
night at 3:14 a cleaning program called **purged** tidies the disk, and
tonight it has decided the tidiest disk is an empty one: its charges are
planted all through the system, set to wipe everything at four o'clock.
Dot included!

In here every bomb is a kill. Drop one, run, BOOM: the blast rips out in a
cross, and one bomb sets off the next in a chain that tears the arena apart.
Kick them, punch them, throw them, lay them in rows, set them off by remote.
Push crates for cover, hold switches down to open the gates, hunt down keys
and lost parts. Ride the beasts that hatch from the eggs the baddies leave.
And watch the kills that hang: they lie there live, waiting.

Act 1 of the Adventure, 27 stages, Dot alone or Dot and Caret together:
through the Screensaver to the Bramblehog, into the Clock, where purged winds
the time on toward four and the Cuckoo waits to strike the hour, and down into
the Buffer, where blocks come back as fast as you blow them away and purged
itself waits at the end. Battle takes up to four players
through six arenas, with revenge riders, sudden death and the dreaded skull.
A love letter to the 90s bomb-arena games.

If you see me running, try to keep up!

## Play

With [av](https://asciiverse.sh), the Asciiverse client:

```
av install uxo
av play uxo
```

To build it yourself: `cargo run --release`.

## Keys

| | Pad | Keys (W A S D player) | Keys (arrows player) |
|---|---|---|---|
| move | stick, d-pad | W A S D | arrows |
| bomb | A, X | Space | Enter |
| ride (hop, charge) | Y | E | . |
| action | B | F | / |

**Action** throws what you carry, else punches the bomb ahead, else picks up
the bomb underfoot, else sets off your oldest remote bomb. Walk into a crate
and lean on it a moment to push it. Start or P pauses (in the Adventure, bomb
then starts the stage over, for a life),
`+` and `-` zoom (`0` automatic), Esc or Select goes back to the title. Every
gamepad is its own player. On the title, left and right on Continue pick
any stage you have reached.

Session log for bug reports: `RUST_LOG=debug uxo 2>uxo.log`

## Licence

MIT; see [LICENSE](LICENSE).
