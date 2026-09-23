# The material economy — the grilling record (C9)

This is the plan for taking §7.4's declared effects from *computed* to *conserved*:
a world where nothing is built from nothing, materials arrive by effort and
processing, tools are made in a lineage, and the citizens can found the
institutions that let any of it happen. It is the arrival of what the founding
record deferred as **"C3's industry supply chains"** (`docs/GRILLING.md`,
*Explicitly open*), and it reopens two decisions taken there.

Everything in **Round 1** below is verbatim from the answers. Where an answer
went against the recommendation, it is marked **overridden** rather than
quietly folded in — the same rule the founding record uses.

---

## What prompted it

> "each material and resource builds from something, similar to the openpbr
> design ideas, we can't make something out of nothing, so houses can't be built
> without their materials, their materials can't be obtained without the effort,
> the tools can be crafted in a lineage tree similar to spore's cell stage. The
> simulation currently does not account for worldly materials and the size of the
> world is very small, there is no mining for resources, no independant tasks for
> civilians outside of roads and the residential/commercial/industrial"

**The premise was corrected before it was accepted.** Materials *are* accounted
for as vocabulary and record: every structure carries `material_as_built`, a
`MAT-*` ticket claims it, `condition` decays it (§8.24), each kind resolves to
declared `PARTS` with families and levels, and `materials::effects` computes cost,
upkeep, decay, desirability and nuisance. What the sim does **not** do is *spend*
any of it. The precise gap is therefore not "no materials" but:

> **the sim can never make something from nothing because it never checks what it
> made something from.**

The rest of the premise was checked and is true as stated: no mining, no
independent civilian tasks (`CitizenState` is `AtHome | ToWork | AtWork |
ToHome | Unemployed`), and a world whose `Tile` is exactly five fields — one
`terrain`, one `zone`, one `road`, one `building`, one `powered` — on a
`256×256` map that `docs/GRILLING.md` Q19 settled as *"a small city built to
scale"*.

---

## Round 1 — the roots of the chain

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q1 | Base of the chain | ground, imports later as a ledger entry | ✔ **"the world itself and it's resources, no outside help"** — imports are now refused, not deferred |
| Q2 | What is conserved, and does money buy material | part units; credits stop substituting | **overridden: mass** — *"money is still part of the economy and how civilians can pay workers to repair stuff, taxation etc. and paying for housing, maybe a false stock market?"* |
| Q3 | Scope and authority | own campaign, overrides written down | ✔ **own campaign**, and *"the agents should be able to slowly work on the world themselves through the system - through the in-game government - and establish it if it doesn't exist"* |
| Q4 | What effort is | citizen work-time first, machines later | ✔ **delegated** — *"best guess derived from the systems outlined"* |
| Q5 | The bootstrap | a declared starting endowment | **overridden: hand-work** — *"the first initial person should get to work as if they were dropped in the new world for the first time"* |
| Q6 | Does a resource deplete | finite deposits, renewables for water/organic | **extended: real timings** — *"not just mine iron ore, smelt in furnace and boom iron ingot, there should be a processing stage that mimicks real world systems"* |
| Q7 | What a lineage node is | a structure that makes the tool | **extended: equipment** — *"a tool is something that the citizens themselves can use to speed up processes… an axe made of metal obviously has extremely strict manufacturing requirements, unless they're fashioning a hatchet from natural materials"* |
| Q8 | Independent civilian tasks | obligations filed into the case-queue | ✔ **delegated with two constraints** — *"the citizens do not require the player to progress, and vice versa, but citizens themselves cannot act out massive goals in one step or by themselves"* |
| Q9 | Map grows or deepens | deepen the tile with layers first | **overridden: chunked** — *"c, maybe using something like OpenCubicChunks/CubicChunks as a bit of a reference for how chunks should be made"* |

### Settled

* **The world is the only source.** No imports, no outside help. The map's own
  material is the ledger, which is what makes the conservation claim checkable at
  all: extracted + produced − spent = stock has to balance against the tiles.
* **Mass is the conserved quantity**, and money is orthogonal to it: money pays
  **labour and services** (repair work, taxation, housing), not material.
* **This is its own campaign**, whose first structural ticket is the tile layer
  from §8.35.
* **The founder.** One person is dropped into the world and works from there; the
  first material is not endowmed by the player, it is done by hand at a bad rate.
* **The player is a proposer, not a place-er**: an all-powerful god who *proposes
  things into the world through the system*, rather than acting directly on it.
* **Processing is real-world-shaped**, multi-stage, with depletion on real-world
  timings — ore does not become an ingot in one hop.
* **Tools are equipment** that citizens use to speed processes, and a tool's own
  manufacture is *strict* in proportion to what it is made of: a natural-material
  hatchet is cheap, a metal axe is a manufacturing chain.
* **Two independent playstyles**: citizens never require the player to progress
  and the player never requires the citizens.
* **Chunked world**, referencing CubicChunks.

### Carried forward unresolved

* **"Atomic habits" plus "without the atoms"** — the granularity of habit-like
  repetition and of real construction *without* simulating atoms. Round 2 asks
  what a task is, which is where this has to become concrete.
* **"A false stock market"** — offered as a maybe. What it trades, and what
  "false" is protecting, is not yet said.
* **Population** — with no outside help and one founder, growth cannot come from
  migration. There is no birth, food or death anywhere in the codebase today.
* **The government's bootstrap** collides with a live rule: `LoadState::FailedClosed`
  refuses every operation *"while the governor is unreadable"*, and a test named
  `a_missing_governor_is_a_refusal_not_a_default` asserts it. Q3 asks the agents
  to found what that rule says cannot exist.
* **Q18 of the founding round** (*"the user should be able to build asynchronously
  and work on their own goals away from the agents"*) and **Q19** (*256×256, ~5k
  agents*) are both touched by these answers and are not yet formally overridden.

---

## Round 2 — the frontier round 1 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q10 | Is the world 3-D | 2.5-D, (b) as a chunk-size change | ✔ **and sharpened**: *"the scene is 3d, the terrain is 3d, but the logic should be 2d, with intent for moving between layers"* |
| Q11 | What a chunk owns | geology chunk-granular, structures entity-granular | ✔ **(b)** |
| Q12 | Mass of *what* | substance table mapping to the family it presents as | ✔ **(a)+(c)**, processes are declared mechanisms with numbers |
| Q13 | Where a process lives | a structure, with the work a citizen task | ✔ **(a)**, and *"the processes themselves are gated behind the progressions, allowing citizens themselves to move up the tech tree through their work"* |
| Q14 | Timescale | real ratios, declared world-scale factor | ✔ **(c)**, and *"how long should an iron deposit last a player - for a while, at least two real-life days"* |
| Q15 | Food and death | produced food, soft needs | **extended to both** — *"animals as well carry farm, farms can have crops… there are death and disease for the farms the animals and the civilians just like the materials rot and decay over time"* |
| Q16 | Founding the government | founding is an always-permitted op | ✔ **referred to `GOD_AGENTS.md`** — *"for a general idea of how that's supposed to go down without conforming yourself to that file"* |
| Q17 | What money buys | labour and services; material trades only inside | ✔ **(a)+(b)+(c)**, and *"citizens can buy and sell things that are not so good"* |
| Q18 | The player's verb | proposals never refused, but must be built | ✔ **(a)+(c)**, and *"the citizens should be allowed to not hard lock in on a task if it's not possible, delegating it for future work that becomes easier as the tech tree expands"* |
| Q19 | What a task is | nested specification → tasks, work on the task | ✔ **(a)**, *"with cookie clicker inspired design for the civilians - so there is real verifiable work done atomically on each task"* |

### Settled

* **Logic is 2-D; presentation is 3-D.** Terrain is drawn in three dimensions and the
  simulation stays two-dimensional, with movement between layers named as an *intent*
  rather than built now. This is what keeps the road graph, the power path and every
  A\* from acquiring a third axis before anything needs it.
* **Chunk ownership split:** geology and surfaces are chunk-granular; structures and
  networks stay entities, because they already carry ids, claims, condition and tickets.
* **The substance table** maps every substance to the family it presents as, and every
  process is a declared mechanism with a number — openpbr's rule applied to matter.
* **A process is a structure**, and the work at it is a citizen's task. **Processes are
  gated behind progressions that citizens advance by working.**
* **Real ratios, a declared world-scale factor**, and an iron deposit that lasts a player
  **at least two real-life days**.
* **Crops, animals and citizens all live under the same rules as materials**: they are
  produced through real stages, and they rot, fall ill and die.
* **Money is a claim on labour and services**; material changes hands only inside the
  world; and a **claim market** is permitted *as a claim market*. Goods exist that are
  *not so good*.
* **The player proposes and is never refused, but the world still has to build it** with
  real material, effort and time.
* **A citizen may refuse to hard-lock on an impossible task** and defer it for work that
  the progression later makes easier.
* **Tasks are atomic, nested and visibly verified** — Cookie-Clicker's feel, the record's
  discipline.

### The founding idea, taken generally from `GOD_AGENTS.md`

Not adopted as binding, as instructed. The general shape it gives:

* **Authority is a stack, not a flag** — Layer 0 constitutional authority, 1 statute,
  2 appropriations and budget, 3 regulation, 4 agency procedures, 5 the individual
  transaction — and the highest controlling authority for a transaction is identified
  *separately* from the lower-level documents that implement it.
* **Authorization is not funding** (§4.7 rule 4), **acceptance is a test that passed**
  (rule 10), **conflicts are preconditions** (rule 7), and **uncertainty is escalated
  rather than invented away** (rule 9).
* **"Founded" is a defect unless an explicit founded record exists** (§7.8).

Four of those already have game mechanics waiting for them: rule 4 is exactly the
*permitted but unfunded* case that Q18's deferral is for, rule 7 is §7.4's placement
precondition, rule 10 is Q19's verifiable atomic work, and rule 8's immutable provenance is
the season record, `MAT-*` claims and retirements.

### Carried forward unresolved

* **The deposit arithmetic contradicts itself** and round 3 puts the numbers on it.
* **How many authority layers the game models**, and what the founding *act* is.
* **The progression's currency and owner** — a city pool, a citizen's skill, or both.
* **Quality on goods** — whether an ingot carries condition, and whether a stockpile rots.
* **Death's failure model** — a loss that files a case, or a loss that ends a run.
* **The deferral's home** and its blocker vocabulary.
* **Where the mass ledger lives** and what proves nothing was invented.
* **Whether 256×256 still binds** — Q19 of the founding round said *"a small city built to
  scale"*, and a chunked world with infinite horizontal tiling is not that.

---

## Round 3 — the frontier round 2 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q20 | The deposit arithmetic | the site is the unit of depletion | ✔ **(c)** |
| Q21 | The founding act and the authority layers | layers real; fail-closed narrowed, not deleted | ✔ **(a)** — and the existing test is *corrected by record*, not removed |
| Q22 | The progression's currency | a city pool **and** per-citizen skill | ✔ **(a)** |
| Q23 | Quality on goods and rot | condition on substances and parts, propagating | ✔ **(a)** — *"so food supply chains have to be very particular"* |
| Q24 | Death's model | a loss with a cause that files a case | ✔ **(a)** — *"deathcare is a service that the citizens can employ and build out"* |
| Q25 | The deferral's home | the case-queue, with the blocker named | ✔ **(a)** |
| Q26 | The mass ledger | mass on every object, plus a ledger `verify.exe` reads | ✔ **(a)** |
| Q27 | Does 256×256 still bind | it stands; chunks are the streaming mechanism | ✔ **(a)** |

### Settled

* **Depletion is spatial.** A deposit occupies tiles; tiles are worked out; the mass behind
  each tile is real-scale so the ratios stay honest, while the *exhaustion a player can
  reach* is the pit rather than a number three orders of magnitude away. This is what
  makes round 1's Q1 answer literal: the map's tiles are the ledger.
* **The authority stack is real.** Layer 0's compact is a record; citizens climb to laws
  and budgets; the existing profiles and `Op`s are layers 3–5. **Fail-closed narrows to
  "the layer governing this op is unreadable"** rather than "no operations at all" — a
  correction to `a_missing_governor_is_a_refusal_not_a_default`, recorded as a correction.
* **Progress is two things**: a city-wide insight pool citizens fill by working, and a
  per-citizen skill that makes that citizen faster at what they have practised. **A death
  must not take a city unlock with it.**
* **Quality is condition**, on substances and parts as well as structures, propagating
  through processing — an ingot that is not so good makes an axe that is not so good. And
  **stored goods rot** on their own declared rate, which is what gives storage a purpose.
* **Death is a loss with a readable cause** that files a case and does not end a run, and
  **deathcare is a service citizens can employ and build out** — so dying is content, not
  just a debit.
* **A deferral is a case-queue entry whose blocker is named** — missing material, no
  process, no skill, unauthorized, unfunded (round 3's Q21 gives that last one its layer).
* **Mass is a field on every object** *and* a ledger per substance that `verify.exe` reads
  back as claims — conservation by construction, audited by a test that fails when it is
  false.
* **256×256 stands** as the shipped and scored area; chunks are how it streams and how it
  grows later.

### Carried forward unresolved

* Which substances the world declares, and which processes exist at what grain.
* How work is *assigned* — who decides what a citizen does next.
* The yield, energy and time numbers behind each process.
* How much mass one tile of deposit holds, and how deep a deposit goes.
* The service taxonomy, now that deathcare is one.
* How food's chain earns "very particular" — preservation, spoilage and the cold chain.
* The lineage tree's shape: tiers, branches, and how a person sees it.
* Price formation — declared prices, or emergent from citizen trade.
* What the player's *propose* verb actually looks like on screen.
* The `v2 → v3` migration, and what geology is derived from for an existing save.

---

## Round 4 — the frontier round 3 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q28 | Which substances exist, at what grain | ~20 substances, ~25 processes, declared as tables | **overridden: (c) is the goal** — full chemistry, alloys and grades, reached incrementally |
| Q29 | Who decides what a citizen does next | one public queue, needs taking priority | **(b) primary** — *"so citizens have lots of autonomy but with a"* — **the answer is truncated; round 5 completes it** |
| Q31 | Mass per tile | declare the scale factor and derive it | ✔ **(a)** — tile size, per-substance density and seam depth are the declarations |
| Q32 | Services and who pays | a fee-and-coverage list, funding declared per service | ✔ **(a)+(b)+(c)** — tax-funded, fee-funded or both, declared per service |
| Q34 | Where prices come from | declared base prices, trade moving them in bounds | **overridden: (b)** — *"using a gold standard, base 100 system"* |
| Q35 | The player's propose verb | a tender: a goal with a site, showing its blockers | ✔ **(a)** |

### Settled

* **Full chemistry is the destination** — alloys, grades, chemistry — reached incrementally from
  a declared table. Nothing in the schema may assume the ~20-substance set is the end state.
* **Citizens have a lot of autonomy** (Q29), with an addition still to be stated. Whatever the
  addition is, it is the government-shaped part of round 1's Q3 answer.
* **The world declares its own scale**: a tile size, a density per substance, a seam depth — and
  the mass in a tile of deposit is *derived* from those rather than declared beside them. At 8 m
  a tile is 64 m²; a 2 m seam is 128 m³; at 2.7 t/m³ that is ~346 t of ore, ~155 t of iron after a
  realistic recovery, so **one worked tile is worth about a house and a half**. The number that can
  be wrong is the tile size, and it is one visible declaration.
* **Services are building kinds** with a declared fee and coverage, and **each declares its own
  funding** — tax, fee, or both.
* **Money is anchored to a commodity and measured by an index**: a **gold standard** and a
  **base-100** system. This is the strongest interlock in the whole plan: if the unit of account is a
  mass of a *mined substance*, the money supply is bounded by the mass ledger, so "nothing from
  nothing" reaches all the way into monetary policy.
* **The player's verb is a tender**: a goal with a site, becoming a record with a specification the
  government turns into tasks, **showing its blockers** so an idle world is never a mystery.

### Carried forward unresolved

* **The truncated Q29** — the "but with a" addition, which is load-bearing for autonomy versus
  government.
* How the substance schema stays data-driven so alloys and grades are *rows*, not code.
* Who holds the gold, whether a reserve ratio is allowed, and what the index measures and from what.
* Food's preservation rules, now unblocked but unasked.
* The yield, energy and time numbers behind each process — they need the schema above.
* The lineage tree's shape — it needs to know what a processed substance *is* first.
* The `v2 → v3` migration and what geology an existing save derives.

---

## Round 5 — the frontier round 4 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q36 | The rest of Q29 | the owner-of-last-resort queue, with the case queue layered | ✔ **both, as recommended** |
| Q37 | The schema that reaches chemistry | two data tables plus a gate, with a stopping rule | ✔ **(a)**, scoped — *"not trying to recreate every single element in the periodic table down to their actual atoms, but at least a little bit closer to it"* |
| Q38 | The gold standard and the index | gold mined, credits a declared mass, index measured | ✔ **"a with c"**, and *"who holds the gold — anyone, treasury if citizens build it, banks if they need it to invest in the market - fractional allowed"* |
| Q39 | Preservation and rot | substance rot rate, processes, climates, cold needs power | ✗ **not answered — re-asked in round 6** |
| Q40 | The `v2 → v3` migration | geology derived from the seed, reported | ✔ **(a)**, and *"the old saves are irrelevant tbh considering the simplicity of the v2 world"* |

### Settled

* **Autonomy by default; central ownership only where nobody owns.** The government is the
  **owner of last resort** — roads, services, anything unclaimed — and the **case system
  surfaces what citizens cannot do themselves**. Priority and advisement stay soft.
* **CORRECTED (round 6): there was no contradiction in Q29.** Round 5's *"a29: a"* was a typo
  for **a39**, so Q29 was never re-answered. It stands as **round 4's (b) primary** — citizens
  decide individually by need and priority, so they have a lot of autonomy — with the truncated
  addition answered by Q36: **a central queue for what nobody owns, and the case system layered
  on top.** The reading is therefore *autonomy by default, central ownership only where nobody
  owns*, and the earlier "reconciliation" in this document was a misreading of a typo. It is
  recorded here rather than deleted, because a document that quietly fixes its own errors cannot
  be audited.
* **The schema is data-driven, and the chemistry is deliberately partial**: two tables plus a
  gate, with the stopping rule — *a process exists when it has a mechanism with a number and a
  consumer* — and the scope stated as *"a little bit closer to it"* than the ten appearance
  families, not a periodic table.
* **Institutions are things citizens build.** The treasury exists *"if citizens build it"*; a
  bank exists *"if they need it to invest in the market"*; **anyone may hold gold**; and
  **fractional reserve is allowed**, which is what makes a run possible and therefore makes the
  money real rather than nominal.
* **The standard can degrade into fiat** ("a with c"), which is historically what happens and
  gives the market dynamics instead of a fixed rule.
* **Old saves are explicitly not a priority** — the v2 world's own simplicity makes the migration
  cheap, so it is planned last rather than designed around.

### Carried forward unresolved

* **Preservation and rot** — asked, unanswered.
* The lineage tree's shape, and what it is a tree *of*.
* The yield, energy and time numbers behind each process, and where each is sourced.
* The citizen-task taxonomy: what a task is as data, and when it is verifiably done.
* What a suspension of the standard *is* mechanically, and who is liable when a run happens.
* **Whether the RCI demand model survives** — the new economy means growth may be
  citizen-driven, and today's model is explicitly *"tuned against the C1 playtest, not asserted
  as correct"*.

---

## Round 6 — the frontier round 5 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q39 | Preservation and rot (re-asked) | substance rot rate, processes, climates, cold needs power | ✔ **(a)** — and the earlier *"a29: a"* is corrected: it was **a39** |
| Q41 | What the lineage tree is a tree of | a capability DAG, tiered and branched | ✔ **(a)**, with a sharpening: *"the metal axe is not a requirement itself but allows better work for longer periods, and a saw blade allows machines to cut trees"* |
| Q42 | Where the process numbers come from | real ratios with provenance, declared for game-facing ones | ✔ **(a)** — a source table is accepted as a real cost |
| Q43 | What a task is as data | a closed verb list, checked for completion | ✔ **(a)**, verb list closed like `Op`'s |
| Q44 | What a suspension is, and who is liable | automatic trigger with declared legitimization | ✔ **"(a) but b, so citizens and the government are forced to work with what they have"** |
| Q45 | Does the RCI model survive | demand demoted to a readout, growth citizen-driven | ✔ **(a)** — `grow()`'s creation of buildings from nothing is to be replaced |

### Settled

* **Citizens are autonomous by default** (the corrected Q29), the government owns only what
  nobody owns, and the case system surfaces what citizens cannot do.
* **Tools are not gates, they are multipliers with a lifetime.** *"the metal axe is not a
  requirement itself but allows better work for longer periods, and a saw blade allows machines
  to cut trees"* — so a tool improves rate and durability, and a tool can be the thing that lets
  a **machine** work at all. Which introduces a distinction the plan did not have: **a tool the
  citizen wears and a tool the machine needs are not the same object.**
* **The process numbers carry provenance**: real ratios where a source exists, declared values
  for what only the game can decide, and a source table accepted as a real cost.
* **A task is `{verb, target, quantity, site, requires{process, tool, skill}, work_remaining,
  claimed_by}` with a closed verb list**, and completion is a *check that reads back* — a task
  whose completion cannot be checked is not a task.
* **Food rots, storage has a climate, and cold storage needs power** — so a blackout can spoil a
  city's food and file a case.
* **The standard does not suspend.** Scarcity binds: *"citizens and the government are forced to
  work with what they have"*. Read against round 5's "a with c", the reading on the record is
  **automatic preconditions, scarcity by default, and a nominal regime only as a deliberate
  act** — flagged here because it is a reconciliation of two answers and therefore worth keeping
  an eye on.
* **`grow()` is to be replaced.** Demand becomes a readout of unmet need; buildings come into
  existence because citizens acquired material and built them, through the same task system as
  everything else. This is the plan's largest single deletion.

### Carried forward unresolved

* **What a citizen is** — needs, skills, health, wealth, home, job, life stage.
* **How the population grows at all** — no migration is permitted, so one founder must become a
  city some other way. Still unresolved since round 1.
* **Tools versus machines**: whether a machine has a tool slot, and whether tool condition and
  structure condition are one field or two.
* **The case taxonomy** for the economy, and who may file.
* **The reserve fraction**, and whether a nominal regime is reachable at all.
* The migration, ranked last by your own answer.

---

## Round 7 — the frontier round 6 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q46 | What a citizen is | the full model, replacing the state machine | ✔ **(a)** — *"houselessness should be equivalant to what it is in real life"* |
| Q47 | How a population grows | a small founding party | ✔ **(a), but allow drifters** — so both: the party *and* arrivals |
| Q48 | Tools versus machines | two objects, a machine with a tool slot | ✔ **(a)**, sharpened — *"the machine itself is a tool and is subject to all the same wear and tear and conditions as every other non living thing in the sim"* |
| Q49 | Case kinds and who files | one prefix per failure class, closed, everyone may file | ✔ **(a)**, filing confirmed for all four |
| Q50 | The reserve fraction, and nominal | 20 % declared, nominal only by act | ✔ **(a)** |

### Settled

* **A citizen is a full agent**, and the five-state machine is **replaced**, not extended: being
  at home or at work becomes a *consequence* of needs and tasks. Needs, skills, illness, wealth,
  a home, a job and a life stage.
* **Homelessness is what it is in real life** — a condition with consequences rather than a flag.
  Which is the sharpest statement yet of how this plan differs from the current sim, where a
  citizen without a home is simply sampled as a case.
* **Population grows two ways**: a **founding party** rather than a lone founder, **and drifters**
  who arrive with nothing but themselves. The first makes the first generation possible; the
  second is a deliberate, accepted leak in "no outside help" — a person may arrive, but nothing
  else may, so the material ledger stays closed.
* **One condition mechanism for all non-living things.** A machine is a structure *and* a tool, and
  it wears exactly like a wall or an axe. This is a simpler answer than the two-object reading I
  recommended: there is one wear system, applied uniformly.
* **Cases are prefixed by failure class and the list is closed**, quoted in full when something
  outside it is filed, and **everyone may file** — a citizen's deferral, an institution's run, the
  government's unfunded goal, the player's tender.
* **Money is hard-bounded**: a declared **20 % reserve**, an automatic trigger, and scarcity that
  binds. Nominal is reachable only by a recorded institutional act, never as an automatic escape.

### Carried forward unresolved

* The needs list and what each unmet need *does* — the failure ladder, and its numbers.
* Drifter rules: how they arrive, what pulls them, and whether they can be refused.
* How an autonomous citizen chooses a task (Q29's (b) needs its own follow-through).
* Whether a worn machine gates its throughput, and whether a tool wears while idle.
* Wages, rent and who sets them.
* **The agent budget** — ~5k was settled for five-state agents; a citizen with needs, skills and
  tasks costs more, and nothing has revisited that number.
* The migration, which your own answer ranked last.

---

## Round 8 — the frontier round 7 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q51 | The needs list and failure ladder | five needs, three rungs each, declared | ✔ **(a)** |
| Q52 | What pulls a drifter in | pull-based on capacity, plus declared attractiveness | ✔ **(a) with a little bit of (b)** — so capacity mostly, with some arrival regardless |
| Q53 | How an autonomous citizen picks a task | needs first, then declared utility weights | ✔ **(a)** |
| Q54 | How a machine's wear bites | condition scales throughput, wearing only while running | ✔ **(a)+(b)+(c)** — unified so that *repair is the same system that governs building* |
| Q55 | Wages, rent, who sets them | structures declare, player sets policy only | ✔ **(a) with (b)** — and the early game must work with **no money at all** |
| Q56 | Does the agent budget hold | keep ~5k, make the cost fit, ticketed | ✔ **(a) with (c)** — *"the ticketing system should make this work much more efficient, but without losing the interpolated accuracy"* |
| Q57 | The migration | regenerate from the seed and report | ✔ **every save has its own seed**, so (c) with (a) |

### Settled

* **Five needs — food, rest, warmth, health, shelter — each with three declared rungs**: degrades
  output, then illness, then death. Homelessness is real, so shelter is a need with real
  consequences rather than a sampling category.
* **Growth is pull-based**: drifters arrive when there is unfilled capacity, with *"a little bit
  of B"* — some arrive regardless, which is a deliberate source of pressure rather than an
  accident. This is also the honest replacement for the demand meter `grow()` used.
* **A citizen chooses by need first, then by declared utility** — skill fit, distance, priority,
  ownership — so "why is nobody mining" has a number for an answer.
* **One wear system for everything non-living, and repair is the same system as building.** A
  machine is a tool, a wall is a tool, and a citizen repairing a blade walks the same task path
  as a citizen building a wall. This is the cleanest answer in the whole record: it means there is
  one `MAKE`/`MAINTAIN` machinery rather than one per noun.
* **The ticketing system is the performance mechanism**, not just a record: needs and skills are
  advanced on boundaries and through tickets, while **interpolated accuracy is preserved** — the
  founding round's Q13 (motion interpolates off work done) is explicitly not sacrificed.
* **Every save carries its own seed**, so the world is *derived* from seed + played history, and a
  migration is a regeneration with a report rather than a translation.

### A consequence nobody decided, recorded as a consequence

**Money cannot exist at the founding.** The unit of account is a mass of *mined* gold (Q38), the
reserve fraction is hard (Q50), and nobody has mined anything yet — so the stone-age game has **no
money at all**, and exchange must be direct: barter, obligation and debt between citizens, with
wages and rents being *claims* that only become payable once the metal exists. Your own Q55 answer
says exactly this (*"civilians in the stone age who can barely work together in the first place
don't just immediatley starve because they have no tangible money to trade"*) — so it is recorded
as settled. But nobody chose *how* the pre-money economy works, and **that is the last unasked
branch**: the world needs a barter/obligation mechanism before it needs a bank.

### Carried forward unresolved

* **The pre-money economy** — barter, obligation and debt before any gold exists.
* The save format's shape, now that the world is seed-derived.
* The UI surfaces the economy needs: the tender, the ledger, the queue, the index, the tree.
* Whether a city can die irrecoverably, and what the player does about it.
* The precise contract behind "ticketed but interpolated" — which quantities are exact per tick
  and which are lazy.

---

## Round 9 — the frontier round 8 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q58 | Exchange before any gold exists | an obligation ledger that becomes banking | ✔ **(a)** |
| Q59 | What a save is | seed plus deltas; derivation is already the pattern | ✔ **(a)** |
| Q60 | Can the city die | a loss with a cause report, which ends the run | **overridden: (b)** — never end the run, let the world get *"really messed up"*, and let the player leave to the main menu for a new run |
| Q61 | The ticketed-but-interpolated contract | three classes, lazy updates a function of elapsed time | ✔ **(a)** |

### Settled

* **Before gold there is no money, and exchange is an obligation ledger** — who owes whom, what,
  and since when — settled in kind now and in metal later. It costs nothing extra because it is the
  same shape as everything else here: a ledger that can be audited and that later *becomes*
  banking.
* **A save is a seed plus its deltas.** Geology and surfaces are derived; extractions, buildings,
  stocks, citizens, institutions and the record are stored. This makes **"derived versus stored" an
  auditable list**, which is the discipline the v1 → v2 migration already established.
* **A run does not end.** The world can collapse into a *"really messed up"* state, and the player
  leaves to the main menu when they choose rather than being stopped. This supersedes the question
  I asked, and it is a better answer than mine: it removes the hard dependency between the player
  and the world's survival (round 1's Q8) even in total failure.
* **Three simulation classes, stated as a rule**: *exact every tick* (positions and the interpolated
  work that drives them), *exact on change* (mass and stocks — moved, never integrated), *lazy on a
  boundary* (needs, skills, condition, rot), and **a lazy update must be a function of elapsed time,
  not of tick count**, or a replay stops matching the seed it came from.

### A new branch, opened by Q60 — recorded, not designed

Your answer added something nobody had asked for, and it is too large to treat as a footnote:

> *"allow the world to get really messed up, so potential futures can be enacted with civilians and
> the player working alone to uncover civilizations and use them for their resources, but with an
> extreme difficulty cost, as a result of them allowing their city to die"*

Three claims are in there, and only the first is settled:

1. **Collapse is a state, not an ending.** Settled.
2. **Ruins are resource sites.** Latent in the material table already: `ruin.parts` is a declared
   world part — *"the retired structure's own parts, read at `deep`"* — so salvage has a vocabulary
   waiting for it.
3. **Civilizations can be uncovered.** Unresolved, and load-bearing: if the world *generates*
   pre-existing civilizations, their ruins are a source of material the city did not make, which is
   either a legitimate in-world resource or a hole in round 1's *"no outside help"* — **and which of
   those it is has not been said.**

### Carried forward unresolved

* Whether world generation places pre-existing civilizations, what their ruins hold, and whether
  that is within "no outside help" or an exception to it.
* What survives a collapse: skills, the insight pool, the record, the seed's deltas.
* How "extreme difficulty cost" is expressed as a number.
* Whether collapse-recovery is this campaign or a named later one.
* What a *run* is at the product level — one save, slots, a main menu listing them.
* The UI surfaces the economy needs: the tender, the ledger, the queue, the index, the tree.

---

## Round 10 — the branch Q60 opened

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q62 | Pre-existing civilizations at generation | yes, inside the rule, with a poor yield | ✔ **(a)** |
| Q63 | What survives a collapse | the record whole, ruins by the retirement rule, unlocks kept | ✔ **(a)** |
| Q64 | Is recovery this campaign | a named later one, with the schema slot reserved now | ✔ **(a)**, and *"c much later"* — both a later campaign **and** its own long-horizon phase |
| Q65 | What a run is | a seed plus deltas plus a record; seasons live inside it | ✔ **(a)** |

### Settled

* **World generation places pre-existing civilizations** from the seed, and this is **inside**
  round 1's rule rather than an exception to it: nothing arrives from beyond the map, and their
  mass was always in the ledger. Their ruins hold **processed** material at a **poor yield and an
  extreme cost** — which is what stops looting from skipping the production chain this whole record
  specifies.
* **Collapse preserves the record whole.** Buildings retire into ruins by the existing rule, the
  insight pool survives, gold stays where it fell, stocks rot, and a new party inherits a written
  history and a looted landscape. Collapse is therefore the most replayable state in the game.
* **Recovery is a later campaign with its own long-horizon phase** — but the **schema reserves the
  slot now**, because a save's derived-versus-stored list is written before the ruins exist.
* **A run is a save**: a seed, its deltas and its record, with seasons and campaigns inside it, and
  the main menu lists runs with their state.

---

## The tree, and the order the work has to happen in

> **CONFIRMED by the user.** Shared understanding reached; the eight-phase order below is
> the order; and the seven overrides were approved and are now recorded **in
> `docs/GRILLING.md`, where the decisions they change actually live** — an override filed
> anywhere else is a drift nobody will find.

**The frontier is empty.** Every branch opened by round 1 has an answer, and nothing above is left
assumed. This section is the shape of the whole thing and the only two things that can still move
it: the **implementation order** and the **overrides** it needs.

### What must exist before any Rust

1. **The source table** (Q42) — every real ratio with its provenance, so a yield is a cited number
   and a game-facing cost is a declared one. The repo already runs on this discipline for evidence;
   this is the same table for matter.
2. **The substance and process schema** (Q37) — two data tables plus a completeness gate, with the
   stopping rule *a process exists when it has a mechanism with a number and a consumer*.
3. Both land in `tools/materials/declare.py`, which already regenerates `generated.rs` and already
   has `table_defects()`-style gates to imitate.

### The order

| phase | what | why it is here |
|---|---|---|
| **1. The world substrate** | tile layers and chunked geology; the scale factor (tile size, density, seam depth); seed-derived deposits; the derived-versus-stored list; `v2 → v3` | everything else is measured in mass and placed on tiles, and Q59 made the save's shape depend on what is derived |
| **2. Mass and the ledger** | mass on every object; the per-substance ledger; `verify.exe` read-back | this is the claim the campaign exists to make checkable |
| **3. Processes and making** | substances and processes as structures; the tier gate; **one MAKE/MAINTAIN system** where repair walks the same path as building | it deletes a whole class of future divergence by having one machinery, not two |
| **4. Agents** | the citizen model replacing the state machine; the task chooser; the three simulation classes; founding party, drifters, births | the consumers of every number above |
| **5. Institutions and money** | the obligation ledger; authority layers and founding; the narrowed fail-closed; gold, treasury, bank, reserve, index; the closed case prefixes | money *cannot* precede mining, so it cannot precede phase 1 |
| **6. Progression** | the insight pool, per-citizen skills, the capability DAG, the lineage | it gates phase 3's processes rather than replacing them |
| **7. Food and the couplings** | preservation, climates, cold needing power, rot | the first place two existing systems do work together |
| **8. Growth** | delete `grow()`; demand becomes a readout; growth is citizens building | last, because deleting it early leaves nothing growing while the replacement is unfinished |

### Overrides C9 needs against decisions already on the record

These are **not** part of the plan until they are confirmed as overrides. Each names the decision
it changes and where that decision lives:

* **`docs/GRILLING.md` Q18** (*"the user should be able to build asynchronously and work on their own
  goals away from the agents"*) — **modified**: the player *proposes* (Q35's tender) rather than
  places directly.
* **`docs/GRILLING.md` Q19** (*"256×256, ~5k agents … a small city built to scale"*) — **kept but
  reinterpreted**: the area and the agent budget stand (Q27, Q56); *"small"* now means *deep*,
  because a tile gained layers, mass and substance.
* **`src/gov/mod.rs`'s `a_missing_governor_is_a_refusal_not_a_default`** — **corrected by record,
  not deleted** (Q21): fail-closed narrows to the layer that governs the op.
* **`docs/GRILLING.md`'s *"C3's industry supply chains, external connections, and weather/seasons"*** —
  **arriving early** as C9. The external-connection half is refused outright by Q1; the supply-chain
  half is this document.
* **The four sim effects (§8.24)** — **superseded in part**: conduction and surface speed were
  deferred for want of consumers, and C9 supplies both (a power line is a structure; a tile gains a
  surface family).
* **`World::grow()`** — **deleted** (Q45). It is the codebase's largest violation of the claim this
  campaign exists to enforce.
* **`CitizenState`** — **replaced**, not extended (Q46).

## Round 11 — the implementation blockers

These were asked after the frontier closed, because the first document could not be written
without them. They are **implementation blockers, not design branches** — the tree was already
empty, and these are the cost of turning it into a document.

| # | Question | ➡️ recommendation | ✔ answer |
|---|---|---|---|
| Q66 | The first slice | the shortest chain that makes the tool which multiplies work | **overridden: "the whole thing"** |
| Q67 | The ledger's unit | one canonical unit, integers, density for display only | **overridden: (b)** — per-substance natural units with declared conversions |
| Q68 | Is a tile 8 m | 8 m, and worth confirming rather than inheriting | ✔ **(a)** |
| Q69 | The progression's tiers | five, named by what the economy can make | **overridden: as many ages as possible, stone age to a modern future** |
| Q70 | How far the source table goes | a named short list with rows; declare the rest | ✔ **(a)** |
| Q71 | Reserving the ruins slot | `ruin` declared as a *source* | ✔ **(a), and (c)** — the salvage rules are designed now, not deferred |

### What the two overrides change

1. **The content is unbounded; the schema is not.** "The whole thing" and "as many ages as
   possible" mean the substance and process tables have no fixed end state — which the schema was
   *already* designed for (round 4's Q28 made full chemistry the goal). **The schema is written once
   and generically; the content is rows.** What this does change is the order's *character*: phases
   1–3 stop being a slice you finish and become the **first working rung** — the schema, the stone
   age, and the first tool — with content added continuously from then on. You cannot test steel
   before you can test a hatchet, so the rungs are still ordered; there is simply no last one.
2. **Salvage comes into scope.** Q71's (c) means the recovery campaign's first half — the rules for
   what a ruin yields, at what cost, from whom — is designed now rather than later. That moves one
   of Q64's deferred pieces forward, and it means the `ruin` source is not a placeholder but a
   working part of the source vocabulary.

### The one reconciliation, flagged rather than assumed

**Per-substance natural units (Q67's (b)) are accepted, and they put the campaign's core claim at
risk — so here is the mitigation, stated as a reading that is overridable like any other.**

The claim this whole campaign exists to make checkable is *nothing from nothing*. A ledger in
litres and tonnes and m³ can only balance if every conversion is exact, and float conversions are
where a balance silently stops balancing. So:

* **every conversion is a declared rational factor** — integer numerator over integer denominator —
  never a float constant;
* **every process declares its unit per input and per output**, and the completeness gate checks
  *dimensional consistency*: a process that mixes litres into kilograms without a declared density
  is a printed defect, not a rounding error;
* **the ledger balances per substance in that substance's own unit**, which is what makes (b) worth
  choosing: a water ledger in litres and a timber ledger in m³ are each readable, and neither is
  pretending to be a mass.

If that is the wrong reading, the honest alternative is (a) — one canonical integer unit with natural
units as display — and it can still be adopted at the schema stage, where it costs nothing.

---

### What this record deliberately does not decide

* The UI surfaces the economy needs (the tender, the ledger, the queue, the index, the tree) — the
  founding round settled that the design doctrine governs the interface, and these need the schema
  in front of them before they can be laid out.
* Collapse-recovery's own design (Q64), beyond the schema slot it reserves now.
* Any number that only a playtest can set — balance, extraction rates, the founding party's size,
  the first child's timing. Those are declared, then measured, then re-tuned by recorded correction,
  which is the pattern §8.24 already followed.

---

## Round 12 — `grow()` deleted, and the task system that replaces it

Round 11 ended on six implementation blockers. Four of them turned out to be the same question wearing
four hats, and the fourth answer changed the *shape* of the plan rather than its order.

### Q72 — what `grow()` becomes, now. Answered: **deleted now**, and the task system is what builds.

Not an interim `grow()` that pays for what it makes. The answer was that this is a game, so the task
system has to exist and be the thing that builds — which made the blocker the phase, not a detour
around one. `grow()` is **deleted from the source**, with a note where it stood naming why, so the next
person to grep for it finds the decision rather than a hole.

What stands in its place is three functions with three jobs, and the split is the point:

* `post_demand_tasks` opens **work** where the city is powered, connected and wants a building;
* `claim_tasks` lets an idle resident take the nearest piece of it — the government as *owner of last
  resort* for work nobody owns (Q36);
* `work_tasks` turns a counter that moves **only while somebody is standing on the site** (Q19's
  atomic, verifiable work).

Nothing raises a structure except finishing that work, and completion draws the plan the task was
opened with — so a structure's provenance is what the plan promised, not a second search that could
disagree with the hole it came from.

### Q73 — where the material comes from. Answered: **nearest matching family.**

Tiles are searched outward from the site for a deposit whose declared family is the structure's own
claimed family: a ceramic home takes stone, sand or clay and never iron ore. Ties break by tile index,
because two equidistant tiles chosen differently on two runs is the quietest way to lose a replay. The
search does not consult the surface — *where a mine may be sited* is the task system's business and not
this function's — and that is written down rather than implied.

### Q74 — what a refusal does. Answered: **files a case.**

A build the world cannot supply is **not posted at all**, and the shortage is recorded aggregated by
district and family: four hundred blocked builds in one district read as **one** case with a count,
the same dedupe the queue already applies. A case carries what was wanted, what was found and how much
is missing, because a refusal a reader cannot size is a refusal nobody can act on. Work posted is not a
building — that assertion is in the test, because it is the whole difference from `grow()`.

### Q75 — does a structure stay tied to its tiles. Answered: **yes, tile lineage.**

`Building.material_from` records the tile, substance and grams of every draw. Empty means a placement
that made no material claim — the raw `place_building` primitive, which the audit makes *visible*
(production zero lineage against a non-zero derived mass) rather than reporting as a defect, because the
audit has one job and it is mass. Lineage **present and disagreeing** with the derived mass is a defect.

### What kind of city this made, and the test that says so

A test used to pin `grow()`'s violation as a measured number: a grown home plus its power plant was
**1,300 tonnes** of material the ground never gave up. The same scenario now runs through the task
system and the audit reports the city **made of what it dug**. Writing it caught a real distinction worth
keeping: the plant and home that test placed directly were themselves unpaid material, and the audit
correctly refused to conserve until they were built through the same path as everything else.

### Two readings that arrived while writing this, recorded rather than smoothed over

* **A 32×32 world holds exactly one substance.** A geology cell is 32 tiles and the field is constant
  within a cell's kind, so a 32×32 world is one patch — seed 7's is iron everywhere and holds no
  ceramic at all, which is why a *ceramic* home cannot be built on it. That is the family rule working,
  and the world is now the test *for* that rule rather than a general fixture. At the game's own
  256×256 the same seed holds ceramic in 4,676 tiles, metal in 1,326 and soil in 3,834, and a test
  asserts every structure's own family can be supplied at game size. **Tuning question it opens,** not a
  decision: whether 128-tile patches over a 256² world give a city enough distinct materials, or whether
  `GEOLOGY_CELL` wants to be smaller than a patch.
* **Material is drawn but not hauled.** The plan names the tiles and the mass leaves them, and nothing
  walks to the deposit: a citizen carries the structure's material to the site in one transaction. Q75's
  lineage is exactly what makes hauling addable later without rewriting the claim — but *today the
  transport step does not exist*, and saying so here is cheaper than letting a reader assume it does.

### Still open, and named

* **The player's own build still pays in credits, not material.** `place_building` is the raw primitive
  the world state uses; routing the player's build through `build_with_material` is a small change with
  one real question attached — what a player does when the city cannot supply the material, which is the
  same case Q74 answers for the sim.
* **Claiming is nearest-first, not utility-scored.** Q53 answered that citizens choose by declared
  weights (skill fit × distance × priority × ownership); that needs the citizen model (Q46) before there
  are skills to score. The placeholder is marked as one in the source rather than left to look like the
  answer.
* **Extraction is instantaneous once planned.** Effort currently enters as *work on the structure*, not
  as time spent getting the material out of the ground.
