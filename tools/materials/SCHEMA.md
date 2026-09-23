# The substance and process schema (C9, Q12/Q37/Q66/Q67)

> **Built, 2026-09-23.** The two tables and the gate now exist: `declare.py` carries
> `TIERS`, `SUBSTANCES` (16 rows) and `PROCESSES` (7 rows, the stone rung); `schema.py`
> is the gate; `emit.py` runs it and **refuses to write** while it reports a defect;
> and `src/materials/schema.rs` is the runtime's typed view with the same checks
> mirrored at the point of use. `SCHEMA_OPEN` carries what the tables do not decide
> into the game's own report. What is *not* built is the second half of phase 3 — the
> MAKE/MAINTAIN machinery that runs a process, and the process structures themselves
> (`VOCABULARY["structure"]` is deliberately empty, which is a declaration rather than
> an oversight). See the build note at the end of `docs/GRILLING-C9.md`.

The campaign's claim is that **nothing is made from nothing**, and this document is where that
becomes checkable rather than asserted. Two data tables, one gate, and one worked chain proven in
exact integers.

The tables live in `tools/materials/declare.py`, beside `PART_PRICE` and the rest, and regenerate
`src/materials/generated.rs` the way everything else there does. This document is the contract; that
file is the data.

## What the schema must make true

1. Every substance has a **unit**, and every quantity converts to grams **exactly** — never by a
   float constant.
2. Every process **balances**: what goes in equals what comes out, in grams, to the gram.
3. Every process has a **mechanism with a number** (Q37's stopping rule): heat, time, labour, power.
4. Every substance either has a **consumer** or is a declared end product. A substance nothing uses
   is a defect, not a placeholder.
5. **The content is unbounded.** Q66 and Q69 said the whole substance set and as many ages as
   possible, stone to a modern future. So nothing here may assume a closed set: a new age is rows,
   and adding one must not require touching the schema.

## The unit rule, and why it is rationals

Q67 chose **per-substance natural units** over one canonical unit, so water is litres, timber is
kilograms, a hatchet is a count. That is honoured — and it is also the one answer in eleven rounds
that put this campaign's central claim at risk, because a ledger in mixed units balances only if
every conversion is exact.

So conversions are **declared rationals**, and the gate refuses anything else:

| Unit | Canonical | Needs | Example |
|---|---|---|---|
| `Mass` | grams | nothing | `timber` is kg → 1 kg = 1000 g |
| `Volume` | millilitres | a declared **density** (g/mL) | `water` is L → 1 L = 1000 mL |
| `Count` | units | a declared **unit mass** (g) | `hatchet` is 1, at 2900 g |
| `Gas` | millilitres | a declared **density** at STP | later; not needed by the first rung |

Every unit-capable quantity is stored as `(numerator, denominator)` integers. `python`'s
`fractions.Fraction` does the arithmetic at declaration time, so the balance is **exact** and the
generated Rust holds integers rather than floats. **A float anywhere in this file is a defect**,
because that is precisely where a balance stops balancing without saying so.

## The two tables

### `SUBSTANCES`

```text
name            the identifier; a substance, not a family (`iron_ore`, not `metal`)
family          the appearance family it presents as (one of the ten)
hue             the anchor it presents with, since a material is family:hue:level
unit            Mass | Volume | Count | Gas
density         g/mL as a rational — required unless unit is Mass or Count
unit_mass       g as a rational — required when unit is Count
rot             condition lost per sim-day as a rational — 0 for anything that does not rot
tags            declared, closed: fuel, edible, tool, structure, precious, salvage
source          how it enters the world: mined | grown | gathered | salvaged | made
```

`source` is the slot Q71 asked to reserve, and **`salvaged` is a working entry, not a placeholder**:
a ruin is a source in exactly the sense a tile is, which is what keeps the pre-money ledger and the
future ruin economy speaking one vocabulary instead of two.

### `PROCESSES`

```text
name            the identifier
tier            the age it belongs to; a process in no tier cannot be reached
inputs          [(substance, quantity as a rational)]
outputs         [(substance, quantity as a rational)]
mechanism       { heat_c, hours, labour_hours, power_kw } — the number Q37's rule demands
requires        { tool, skill, structure } — what must exist for the work to be possible
waste           the declared losses, which are outputs and therefore balance
```

A process with no `mechanism` is a defect. A process whose tier does not exist is a defect. A
process whose inputs or outputs cannot become grams is a defect.

## The gate

`table_defects()` already exists for the effects tables (§8.24) and this is its sibling. It prints,
and the run fails, when:

* a substance has no unit, or a `Volume` with no density, or a `Count` with no unit mass;
* a process fails to balance in grams — **and the defect quotes both totals**, because
  `in 2995 g vs out 2990 g` is a sentence someone can act on and "unbalanced" is not;
* a process names a substance, family, tier, tool or skill that does not exist;
* a substance has no consumer and is not a declared end product;
* **a process mixes dimensions without a declared conversion** — this is the gate Q67's risk
  demanded: litres into kilograms with no density is refused, and a density that disagrees with the
  substance's own declared density is refused too, so a process cannot invent a conversion it needs;
* any quantity or density in the tables is a float rather than a rational.

## The worked chain

The stone rung, and it is deliberately the one Q7 named — *"fashioning a hatchet from natural
materials"* — because it is the shortest chain that ends in a tool which multiplies work. Masses in
grams, quantities as exact rationals, and every line balances on its own.

| step | tier | in | out | balances |
|---|---|---|---|---|
| gather `timber` | hands & stone | *(from the world)* | 3000 g timber | source |
| **riven haft** | hands & stone | 3000 g timber | 2400 g haft blank, 600 g offcuts | 3000 = 3000 |
| gather `stone` | hands & stone | *(from the world)* | 1000 g stone | source |
| **knap a core** | hands & stone | 1000 g stone | 800 g knapped edge, 200 g flakes | 1000 = 1000 |
| gather `plant_fibre` | hands & stone | *(from the world)* | 100 g fibre | source |
| **twist cord** | bound & composite | 100 g fibre | 95 g cord, 5 g waste | 100 = 100 |
| **assemble the hatchet** | bound & composite | 800 g edge + 2400 g blank + 95 g cord = **3295 g** | 1 hatchet @ 2900 g + 395 g waste | 3295 = 3295 |

Two things this chain proves rather than claims:

* **The ledger balances at every step, in exact integers.** The hatchet is a *Count* whose unit mass
  is what lets it enter a gram total; a counted good without a unit mass is refused by the gate, and
  that refusal is what stops "1 hatchet" from being a number nobody can audit.
* **`bound & composite` is a real tier with real work in it**, which is the answer to Q69 that
  matters most: without it the stone age has nowhere to be and the first tool is gated behind mining
  metal, making the bootstrap circular.

The ore-to-blade chain is the rung above it and is not written here: it needs the source table's
ratios (Q42/Q70) before its numbers are citations rather than inventions. **The schema does not
depend on it, which is the point** — the schema is written once and the ages are rows.

## Built, and what the gate found on its first run

The gate's first run against the tables found **five defects**, and three were real
things rather than my arithmetic: `sand` and `clay` were mined with no consumer and no
declared reason (the rule made the omission visible), and the balance rule as first
written refused a *gather* — a process whose other side is the world's own ground, which
the worked chain above marks as "source". Both were fixed in the data and the rule
respectively, and the gather case is now stated: a process with no inputs may only
produce what the world already holds, which is the rule that replaces `grow()`.

## What this document does not decide

* **The ages, their names and how many.** Q69 asks for as many as possible; they are rows, and the
  first three are the only ones the first rung needs.
* **The content of any age above the stone rung.** Ore grades, coke ratios and kiln temperatures are
  the source table's job.
* **The runtime shape of the generated table**, beyond the rule that it holds integers and never
  floats. It follows the existing `generated.rs` pattern.
* **How a deposit's mass in a tile is derived** — that is phase 1's scale factor (8 m tile, density,
  seam depth), which multiplies out to the ~346 t the record already states.
