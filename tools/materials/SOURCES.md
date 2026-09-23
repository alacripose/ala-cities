# The source table (C9, Q42/Q70)

Q42 chose **real ratios where a source exists, declared values where only the game can decide**, and
Q70 scoped it: rows for **the short list of ratios that carry the feel**, and declared for the rest.
This is that table.

The rule it follows is this repo's own, from `GOD_AGENTS.md` §11.8: *"a source row registering
content that does not exist is worse than an uncited claim, because it reads as provenance."* So a
number here is either **cited with a row** or **declared and marked as declared** — there is no third
state, and **"not yet sourced" is a legitimate entry that is not a citation.**

Markers: **[V]** read from the source in this session · **[D]** declared, no source claimed ·
**[NS]** not yet sourced, needs a row before it may be used in a yield.

## Read this session

| # | Quantity | Value | Source | Used by |
|---|---|---|---|---|
| 1 | Iron ore charged per tonne of pig iron | **1.6–1.8 t** | marketreportsworld, *Pig Iron market* (secondary) **[V, weak]** | the smelt process's input ratio |
| 2 | Metallurgical coke per tonne of pig iron | **450–550 kg** | same **[V, weak]** | the smelt's fuel input |
| 3 | Coke per tonne of pig iron, older practice | **680–907 kg** (1,500–2,000 lb/ton) | NML India, *Low Shaft Furnace Smelting of Pig Iron* **[V]** | see the divergence below |
| 4 | Hot blast temperature | **700–1,000 °C** | metallics.org, *Pig iron production* (industry body) **[V]** | the smelt's `heat_c` floor |
| 5 | Coke combustion, raceway | **1,000–1,600 °C** | same **[V]** | the smelt's `heat_c` working range |
| 6 | Limestone calcination, material temperature | **~850–900 °C**, optimum ~900 °C | LiU *Numerical Modelling of the Calcination Process*; ResearchGate, *Limestone Characteristics and Calcination Temperature* **[V]** | the lime process's `heat_c` |
| 7 | Cement kiln flame temperature | **1,480–1,650 °C** (2,700–3,000 °F) | USGS, *Heating Limestone* **[V]** | see the distinction below |
| 8 | Clinker formation temperature | **~1,450 °C** | ScienceDirect, *Clinker Production* **[V]** | the clinker process's `heat_c` |

### Two findings, recorded rather than resolved

**1. The coke rate diverges by a factor of nearly two, and the divergence is the finding.** Rows 2
and 3 disagree — 450–550 kg versus 680–907 kg of coke per tonne of pig iron — and they are not
measuring the same thing: row 3 is *coke alone*, row 2 is a modern charge which typically displaces
coke with injected coal or gas. **So the declared value has to say which practice it models**, and
which blast-furnace era the process belongs to — which is exactly the question Q69's ages opened. A
yield is therefore not one number but **one number per age**, and the table's shape has to allow it.

**2. "Kiln temperature" is ambiguous between the material and the flame, and the schema must say
which.** Rows 6 and 7 are both correct and differ by ~600 °C: limestone *calcines* at ~900 °C while
the *flame* that heats it runs at ~1,500 °C. A process declaring `heat_c` without saying whether it
means the material or the flame would produce two different games from the same citation, so
`PROCESSES.mechanism` needs the distinction named rather than implied.

## Declared, and marked as declared

* **Every duration and labour cost** (`hours`, `labour_hours`). No source states what a *game* smelt
  costs, and inventing a citation for one would be the defect §11.8 describes.
* **Every `power_kw`.** Same reason: real furnaces are measured in GJ per tonne, and converting that
  into a game's power units is a declared design choice, not a sourced ratio.
* **All densities and unit masses.** Real values exist and are nearly constant, but a rational
  declaration is required by the schema and a float is refused — so each is declared as a rational
  and cited separately if it is worth a row.

## Not yet sourced — and these block the ore-to-blade rung

| Quantity | Why it is needed | Status |
|---|---|---|
| Iron ore grade (% Fe by deposit type) | the whole extraction-to-metal ratio rests on it | **[NS]** |
| Steelmaking: pig iron + scrap per tonne of steel | the rung above the smelt | **[NS]** |
| Charcoal yield from timber (kg charcoal per kg wood) | the pre-coal ages need it, and it is the bronze/iron era's actual fuel | **[NS]**, and **declared around** since round 28: `char timber` uses 25 %, marked `[D]` in its own note, because a fire is what the founding day has and the row is still unfilled |
| Timber and crop yields per hectare per year | food and the bound & composite tier | **[NS]**, and **declared around** since round 28: `forage` uses 2 kg for half an hour's work, marked `[D]`, because a person has to eat before either row is filled |
| Charcoal/wood heat for a bloomery | the pre-blast-furnace iron path | **[NS]** |

**These need USGS and agronomy/forestry sources, not market summaries**, and they are the next fetch
rather than a declaration — because a declared ore grade would make the whole conservation ledger
look sourced while being invented.

## What this table does not decide

* **Which age a yield belongs to.** Row divergence finding 1 says a yield may be per-age; whether the
  *tables* carry a per-age yield or a per-age process is a schema question the next rung will force.
* **Any density or unit mass as a sourced number.** They are declared rationals for now (above).
* **Whether a declared number may later be promoted to a cited one.** It should be able to be, which
  means a declared row needs the same identity a cited one has.
