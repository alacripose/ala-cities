# C1 — automated playtest record

A session was launched against the release build and its output read back.

**A first draft of this record said "nobody clicked anything". That was false,
and the record is what disproved it.** The session capture holds three sessions
with **9, 11 and 32 recorded interactions**, each carrying a screen coordinate
and a tile — genuine mouse input, because the `act_*` methods have exactly one
call site, the mouse handler. The player was playing the window while these runs
were in flight; that is where the 35 `operator` tickets came from. So this is a
mixed session — a scripted launch with real input — and it is better evidence
than a purely automated one would have been.

The distinction is kept because it matters: what follows establishes the
*engine's* behaviour and the record's integrity. It does not establish that the
interface is usable, and the player's own report on that is in
`docs/GRILLING-C2.md`.

**Build.** `cargo build --release`, tag `c1-sim-core` plus the C1 follow-up
commit. (A real build-identity hash is still open — it wants a build script.)

**Machine.** NVIDIA GeForce RTX 2070 SUPER, Vulkan, `Bgra8UnormSrgb`,
present modes `[Fifo, FifoRelaxed, Mailbox, Immediate]`, window 1600×900
physical, **`scale_factor = 1.0`**.

---

## What was run

| Step | Command | Result |
|---|---|---|
| Tests | `cargo test` | 57 lib + 13 bin pass, 1 ignored (a measurement) |
| Lint | `cargo clippy --all-targets` | 0 warnings |
| Session | `./target/release/ala-cities.exe`, 150 s wall | reached tick 1600, autosave fired, 32 player interactions captured |
| Oracle | `./target/release/verify.exe` | exit **0** — see below |

### What the player did, read out of the capture

| Capture | Interactions | Contents |
|---|---|---|
| `session-1790059495244` | 9 | 2 road legs, 6 zone paints, 1 service placed |
| `session-1790059938438` | 11 | 3 road legs, 4 zone paints, 2 services, 1 demolition |
| `session-1790060024253` | 32 | 19 road legs, 6 zone paints, 5 services, 1 demolition |

Every one of those is a file in `playtest/C1/sessions/`, and every one names the
ticket it filed. That is the loop running end to end: a click, a ticket, a world
change, and a verdict read back out of the world afterwards.

## What the session produced

```
record — saves\season_2026_s1
governor — v3 profile `default`
tickets — 42 filed, 4 open, 38 terminal
evidence — 35 · retirements 38 · corrections 0 · recorded refusals 0
world — saves/world.ron   (snapshot at tick 1200)
world read-back — 23 reproduced, 0 contradicted, 8 superseded, 0 late
  not checkable — 4 claim(s) were closed after this snapshot
```

The record is a real one: tickets, per-ticket evidence files, retirements with
reasons, an append-only `events.jsonl`, and a world snapshot that the verifier
reads the verdicts back out of.

## Measured numbers

| Thing | Value | How |
|---|---|---|
| World snapshot size | **4,534,506 bytes** | `saves/world.ron` on disk |
| Snapshot write cost | **30.7 ms** | `cargo test --lib measure_world_save_cost -- --ignored --nocapture` |
| Snapshot load cost | **125.7 ms** | same |
| Compact vs pretty | 4.49 MB vs 10.8 MB, 31 ms vs 46 ms | both measured, not estimated |
| Body text on panel | **14.24:1** | startup contrast check (AA floor 4.5) |
| Body text on desk | **17.02:1** | same |
| Muted text on panel | **8.57:1** | same |
| Text on ink (primary) | **7.02:1** | same |
| Style tokens | 32 tokens, 32 records, **0 with no record** | same |

## The oracle found the oracle's own bug, then found nothing

The first run of the verifier reported **12 contradictions** on a session nobody
had played. All twelve were false, and each one was false for a different
reason — which is exactly why the trail is kept here rather than tidied away:

1. **8 were supersessions.** A claim was true when written, and a later recorded
   action legitimately changed the tile. `Expectation::tiles()` now lets the
   verifier ask whether a later *terminal* ticket names the same tile, and
   reports those as superseded instead of contradicted.
2. **2 were demolitions** recorded under a later ticket — the same mechanism, and
   the reason the check must be "explained by a record", not "still true".
3. **4 were uncheckable.** The snapshot is at tick 1200; those claims were
   retired at ticks 1228, 1268, 1388 and 1568 — *after* it. A world from before
   the work cannot speak to the work. The verifier now says so, and counts them
   separately.

The mechanism behind most of it: **growth consumes a zone.** `place_building`
clears the tile's zone when the city builds on it, so "24 tiles carry the zone
set" stops being literally true the moment the zone succeeds. That is the zone
working, not a claim failing — and a verifier that cannot tell the difference
teaches its reader to ignore it.

Two further details worth recording, because both were visible in the files:

- `tickets/*.json` is the ticket **as filed**; the terminal state lives in
  `retirements/*.json` and is reconciled on open. Reading only the ticket files
  makes a retired ticket look open.
- The remaining real limitation: with only a periodic snapshot, the read-back is
  a final-state check. Reading a claim against the world *at its own tick* needs
  a replay, and replay is already an open item.

## Known behaviours, stated rather than discovered later

- **The sim runs at roughly half wall speed while the window is unfocused.** The
  render loop is throttled in the background, and the catch-up cap (8 steps per
  frame) then drops sim time on purpose rather than letting the record gain ticks
  that never simulated. 150 s wall reached tick 1600 rather than 3000. In the
  foreground at a normal frame rate this does not bite.
- **Autosave is a hitch, not a stall:** ~31 ms once per sim-month, on the same
  boundary the economy posts on.
- **A hard kill loses the world snapshot only since the last autosave.** The
  record survives either way; that separation is the point.

## Judgement items — NOT established by this run

Listed so a green run cannot be mistaken for these:

- Composite contrast **on a rendered frame** (the token pairs are measured; the
  pixels are not).
- Layout at every window size.
- Frame timing under load.
- **Whether the city is any fun.**

## Playing it

```
cargo run --release
```

| Key | Does |
|---|---|
| `1` `2` `3` `4` `5` | road · zone · power · demolish · inspect |
| `R` `C` `I` | zone type, while the zone tool is active |
| drag | lay a road leg, or paint a zone block |
| right-drag / wheel | pan · zoom |
| `space` / `Tab` | pause · cycle 1x 2x 3x |
| `L` / `H` | ledger · help (the help panel also carries this stage's test script) |
| `Esc` then `F` | leave feedback at any time; Enter writes it to `playtest/C1/` |
| `Esc` then `S` / `O` / `Q` | save world · load world · flush the interaction capture |

The interface scale is logged at startup (`interface scale scale_factor=…`), and
the window is uncapped so a 240 Hz panel is actually fed.

## What the player asked to be brought forward

Recorded here because it becomes round 7 of the grilling rather than a quiet
patch: text that is **hard to read and inconsistently sized**, a scene that is
**2D top-down**, and a UI with **no icons** that does not follow
`UNIFIED_DESIGN.md`. The measured state behind each is in
`docs/GRILLING-C2.md`.
