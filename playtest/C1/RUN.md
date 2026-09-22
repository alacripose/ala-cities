# C1 — sim core · playtest script

This stage is runnable from the repository at any time. The script below is
**scheduled first**: it is what the stage is for, and the capture is watching
for these steps before anything else you choose to do.

```bash
cargo run --release      # the game
cargo test               # 55 invariants, headless
cargo run --bin verify   # re-check the record with no window and no GPU
```

Your interactions are recorded while a test is running — what you clicked, where
on the screen, which tile it landed on, which tool was active, and the ticket it
filed. The record is local, stays in this repository, and is never sent
anywhere. The top bar says `● RECORDING` while it is on, because recording is a
state and the chrome should say so.

**Feedback is available at any time**, not only at the end: press `Esc`, then
`F`, type, then `Enter`. It is written into `playtest/C1/feedback/` with the
session attached and its confidence recorded as `tentative` — one playtest is
one playtest.

---

## The scheduled steps

**1. Connect the city to somewhere.**
The map opens with one strip of road down its western edge and nothing joined to
it. Pick the Road tool (`1`), drag from the edge road to the middle of the map,
and watch the tile preview turn **green** before you release. Release.

*What to look for:* a ticket id appears in the top bar. Press `L`. The ticket is
open. Within a couple of seconds it closes as `completed_and_validated`, with
grade-B evidence, because the world was read back and the road was there.

**2. Zone a residential district.**
Pick Zone (`2`), press `R`, and drag a block of tiles **beside your new road**.
Only tiles touching a road within two tiles can grow.

*What to look for:* zoned tiles tint. They stay tinted and do not grow, because
there is no power yet. That is not a bug — it is the thing the next step is for,
and the city's own ledger has already filed it as a case.

**3. Build a power plant.**
Pick Power (`3`), click a tile beside your road.

*What to look for:* `power` in the left column stops saying `0 plants`. Watch the
tint: powered tiles light, and the tiles your plant's roads cannot reach stay
dark with a small brown marker. Those dark tiles are what the open case is
counting.

**4. Let it run, and then break something.**
Press `Tab` to go to 3x. Houses appear. Citizens appear and walk to work.

Then pick Demolish (`4`) and remove one house.

*What to look for:* the house does **not** vanish. It stays on the map, faintly
drawn. Press `L` — the ledger says how many structures are retired. Demolition
files a ticket too, and the retired building is named inside that ticket's
objective.

**5. Read the ledger properly.**
Press `L` and leave it open for a while.

*What to look for:* the board **drains**. `open` should be small and should not
grow without end. Anything the world never showed closes as
`completed_but_unverified` — never as validated. If you see a ticket closed as
validated with no evidence attached, that is a defect: say so in feedback.

**6. Test a refusal.**
Quit, then run:

```bash
ALA_GOVERNOR=config/governor.read_only.json cargo run --release
```

Try to lay a road.

*What to look for:* the top bar reads `GOVERNOR FAILED CLOSED`… no — it reads
`governor v3 · read_only profile`, and the road is **refused** with a banner that
names the operation *and the vocabulary that would have been accepted*. The city
is unchanged. This is the difference the whole governance layer exists for: a
refusal you can act on is not a dead end.

Then try it once more with a deliberately broken file:

```bash
ALA_GOVERNOR=config/does-not-exist.json cargo run --release
```

*What to look for:* every build operation is refused, and the reason says the
governor could not be read. **A governance layer must not fail open**, so a
missing file denies rather than quietly permitting.

---

## What is deliberately not in C1

Services other than power, milestones and tile unlocks, industry supply chains,
external connections by rail, ship and air, weather and seasons, districts and
policies, audio. Those are C2 and C3, and they are recorded as open in
`docs/GRILLING.md` rather than assumed.

## What this stage does not establish

Whether the city is fun. Whether 240 Hz holds under load on your machine — the
top bar reports frames per second, and the number it prints is measured from
frame deltas, not asserted. Composite contrast on a rendered frame. Whether the
HUD reads well at every window size. Those are judgement items and they are
reported as open by `cargo run --bin verify`, not as passed.

## Reporting a finding

A useful record says **what you did**, **what you expected**, and **what the
world showed instead**. Name the ticket id if there is one. The capture already
holds the clicks; the thing only you can supply is the expectation.
