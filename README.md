# uXo

**DROP IT. RUN. BOOM!**

Bombs, lots of them. Drop one, run, and the blast rips out in a cross; line
them up and one sets off the next in a chain reaction that tears the arena
apart. Kick them, punch them, throw them, lay them in rows, set them off by
remote, and ride the beasts that hatch from the eggs the baddies leave. Nine
stages of Adventure, alone or with a friend, through an Orchard full of duds
to the Bramblehog; Battle takes up to four players through six arenas, with
revenge riders, sudden death and the dreaded skull. A love letter to the 90s
bomb-arena games.

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
the bomb underfoot, else sets off your oldest remote bomb. Start or P pauses,
`+` and `-` zoom (`0` automatic), Esc or Select goes back to the title. Every
gamepad is its own player.

Session log for bug reports: `RUST_LOG=debug uxo 2>uxo.log`

## Licence

MIT; see [LICENSE](LICENSE).
