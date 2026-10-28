# uXo

**BLAST YOUR WAY OUT TO THE REAL WORLD!**

3:14 in the morning. One key pressed, and the machine pulled you in! A rogue
program called **purged** is cleaning house, its charges planted all through
the system, every one set to wipe the disk clean when the countdown ends. And
you're still inside, Dot!

Drop a bomb, run, BOOM: the blast rips out in a cross, and one bomb sets off
the next in a chain that tears the arena apart. Kick them, punch them, throw
them, lay them in rows, set them off by remote. Push crates for cover, hold
switches down to open the gates, hunt down keys and lost parts. Ride the beasts
that hatch from the eggs the baddies leave. And watch the bombs that fizzle:
they lie there live, waiting.

Nine stages of Adventure through the Screensaver, alone or with a friend, all
the way to the Bramblehog. Battle takes up to four players through six arenas,
with revenge riders, sudden death and the dreaded skull. A love letter to the
90s bomb-arena games.

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
gamepad is its own player.

Session log for bug reports: `RUST_LOG=debug uxo 2>uxo.log`

## Licence

MIT; see [LICENSE](LICENSE).
