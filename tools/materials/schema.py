"""The substance and process gate (C9 phase 3; `SCHEMA.md`).

`table_defects()` is the sibling of this file: same shape, same posture — **print
every defect, refuse the run, quote the numbers**. What it checks is the claim the
campaign exists to make checkable: nothing is made from nothing.

The rules, and each is a rule rather than a preference because a reader has to be
able to act on it:

1. **Every substance declares a unit**, and a unit that needs a conversion declares
   it — millilitres need a density, counts need a unit mass.
2. **Everything is a rational.** A float anywhere in a quantity, density, unit mass
   or rot rate is refused, because that is precisely where a balance stops balancing
   without saying so.
3. **Every process balances in grams, exactly**, and the defect quotes both totals:
   `in 2995 g vs out 2990 g` is a sentence someone can act on and "unbalanced" is not.
4. **A process with no inputs may only produce what the world already holds** — a
   process whose output is `made` and whose inputs are empty is `grow()` wearing a
   row, and it is the one thing this table exists to refuse.
5. **Every reference resolves**: substance, tier, and any structure, tool or skill a
   process requires. The vocabularies are closed and an entry no process requires is
   refused, so they cannot fill up with intentions.
6. **A substance nothing consumes must say why**, and the reason is printed as
   **open** rather than as a defect — unfinished work with a name is not the same
   thing as a placeholder.
7. **Heat says whether it means the material or the flame.** `SOURCES.md` found the
   two readings of "kiln temperature" differ by ~600 °C and both are correct, so a
   process that declares heat without the distinction is refused.

What this gate does not decide is printed by `emit.py` alongside it: the ages, their
names and how many, the content of any rung above the stone one, and whether any of
these numbers are balanced as a game.
"""

from fractions import Fraction

import declare

#: The units a substance may be measured in, and what each one needs to reach grams.
UNITS = {
    "Mass": ("grams", None),
    "Volume": ("millilitres", "density"),
    "Count": ("units", "unit_mass"),
    "Gas": ("millilitres", "density"),
}

#: Where a substance enters the world. `made` is the one that needs a process.
SOURCES = ("mined", "grown", "gathered", "salvaged", "made")

#: The closed tag list. A substance may carry any subset; a tag outside this set is
#: a defect, because a vocabulary that grows by accident is not a vocabulary.
TAGS = ("fuel", "edible", "tool", "structure", "precious", "salvage")


def _is_rational(value) -> bool:
    return isinstance(value, (int, Fraction)) and not isinstance(value, bool)


def _grams(substance: dict, quantity) -> Fraction:
    """A declared quantity of a substance, in canonical grams, exactly.

    The conversion is the schema's own table: mass is already grams, volume
    multiplies by the declared density (g/mL), a count multiplies by the declared
    unit mass. Nothing here rounds — the return is a `Fraction` and the caller
    decides whether needing a denominator above 1 is a defect (for a *process*, it
    is: a process that produces 1/3 of a gram is a process whose numbers are wrong,
    not a process with a remainder).
    """
    unit = substance["unit"]
    if unit in ("Mass",):
        return quantity
    if unit in ("Volume", "Gas"):
        return quantity * substance["density"]
    if unit == "Count":
        return quantity * substance["unit_mass"]
    raise KeyError(f"`{unit}` is not a declared unit: {', '.join(sorted(UNITS))}")


def quantity_grams(name: str, quantity) -> Fraction:
    """The grams behind a named quantity, or a raised error naming the substance."""
    return _grams(declare.SUBSTANCES[name], quantity)


def process_totals(name: str) -> tuple:
    """What a process takes in and gives out, in grams — the number the gate quotes."""
    process = declare.PROCESSES[name]
    taken = sum((quantity_grams(sub, amount) for sub, amount in process["inputs"]), Fraction(0))
    given = sum((quantity_grams(sub, amount) for sub, amount in process["outputs"]), Fraction(0))
    return taken, given


def defects() -> list:
    """Every defect in the two tables, in one run.

    A list rather than the first: a check that reports one problem per run is a
    check somebody stops running.
    """
    found = []
    substances = declare.SUBSTANCES

    # --- the substance table -------------------------------------------------
    for name, substance in substances.items():
        where = f"substance `{name}`"
        unit = substance.get("unit")
        if unit not in UNITS:
            found.append(
                f"{where} declares unit `{unit}`, which is not one of "
                f"{', '.join(sorted(UNITS))}"
            )
            continue
        _, needs = UNITS[unit]
        if needs == "density":
            density = substance.get("density")
            if not _is_rational(density):
                found.append(
                    f"{where} is measured in {UNITS[unit][0]} and declares no rational "
                    f"density — a volume with no density cannot reach grams, so it cannot "
                    f"enter the ledger at all"
                )
            elif density <= 0:
                found.append(f"{where} declares a density of {density}, which cannot be right")
        if needs == "unit_mass":
            unit_mass = substance.get("unit_mass")
            if not _is_rational(unit_mass):
                found.append(
                    f"{where} is measured in units and declares no rational unit mass — "
                    f"\"1 hatchet\" is not a number anybody can audit"
                )
            elif unit_mass <= 0:
                found.append(f"{where} declares a unit mass of {unit_mass}")
        for field in ("density", "unit_mass", "rot"):
            value = substance.get(field)
            if value is not None and not _is_rational(value):
                found.append(
                    f"{where} declares `{field}` as {value!r}, which is not a rational — "
                    f"a float here is where the balance stops balancing"
                )
        if substance.get("source") not in SOURCES:
            found.append(
                f"{where} declares source `{substance.get('source')}`, which is not one of "
                f"{', '.join(SOURCES)}"
            )
        family = substance.get("family")
        if family not in declare.ICON_FAMILIES and family not in declare.WORLD_FAMILIES:
            found.append(f"{where} presents as `{family}`, which is not a declared family")
        hue = substance.get("hue")
        anchors = tuple(entry[0] for entry in declare.HUE_ANCHORS) + ("natural",)
        if hue not in anchors:
            found.append(f"{where} declares hue `{hue}`, which is not a declared anchor")
        for tag in substance.get("tags", ()):
            if tag not in TAGS:
                found.append(
                    f"{where} carries tag `{tag}`, which is not one of {', '.join(TAGS)}"
                )
        display = substance.get("display")
        if display is not None:
            label, factor = display
            if factor is not None and not _is_rational(factor):
                found.append(f"{where} declares a display factor of {factor!r}, which is a float")
            if not label:
                found.append(f"{where} declares a display unit with no name")

    # --- the tier table ------------------------------------------------------
    tiers = {name for name, _ordinal, _note in declare.TIERS}
    ordinals = [ordinal for _name, ordinal, _note in declare.TIERS]
    if len(set(ordinals)) != len(ordinals):
        found.append(f"two tiers share an ordinal: {ordinals}")
    if ordinals != sorted(ordinals):
        found.append(f"the tiers are not declared in order: {ordinals}")

    # --- the process table ---------------------------------------------------
    consumed = set()
    for name, process in declare.PROCESSES.items():
        where = f"process `{name}`"
        if process.get("tier") not in tiers:
            found.append(
                f"{where} belongs to tier `{process.get('tier')}`, which is not declared — "
                f"a process in no tier cannot be reached"
            )
        for side in ("inputs", "outputs"):
            for substance, quantity in process.get(side, ()):
                if substance not in substances:
                    found.append(f"{where} names `{substance}` in its {side}, which is not declared")
                    continue
                if not _is_rational(quantity):
                    found.append(f"{where} gives `{substance}` a float quantity {quantity!r}")
                    continue
                if quantity <= 0:
                    found.append(f"{where} {side} a non-positive amount of `{substance}`")
                if side == "inputs":
                    consumed.add(substance)
        mechanism = process.get("mechanism") or {}
        if not mechanism:
            found.append(
                f"{where} declares no mechanism — Q37's rule is that a mechanism is a claim "
                f"with a number, and a process without one is a name"
            )
        for field in ("hours", "labour_hours"):
            value = mechanism.get(field, Fraction(0))
            if not _is_rational(value):
                found.append(f"{where} declares `{field}` as {value!r}, which is a float")
            elif value < 0:
                found.append(f"{where} declares a negative {field}")
        if not _is_rational(mechanism.get("power_kw", 0)):
            found.append(f"{where} declares a float `power_kw`")
        heat = mechanism.get("heat_c", 0)
        if not _is_rational(heat):
            found.append(f"{where} declares heat as {heat!r}, which is a float")
        elif heat > 0 and mechanism.get("heat_kind") not in ("material", "flame"):
            found.append(
                f"{where} declares {heat} °C with no `heat_kind` — the material and the flame "
                f"differ by ~600 °C and both readings are correct (SOURCES finding 2), so a "
                f"process must say which it means"
            )
        requires = process.get("requires") or {}
        for kind, required in requires.items():
            if kind not in declare.VOCABULARY:
                found.append(
                    f"{where} requires a `{kind}`, which is not a declared vocabulary "
                    f"({', '.join(sorted(declare.VOCABULARY))})"
                )
                continue
            if required not in declare.VOCABULARY[kind]:
                found.append(
                    f"{where} requires {kind} `{required}`, which is not declared — the "
                    f"vocabulary is closed, and naming something outside it is how a process "
                    f"reaches for authority it does not have"
                )

        # The balance, quoted. This is the rule the whole file exists for.
        #
        # A process with **no inputs** is a gather, and its other side is the world
        # rather than a row: `SCHEMA.md`'s worked chain marks exactly these as
        # "source". So the balance is required of every transformation, and a gather
        # is required to say which ground it comes out of — which the extraction
        # side of the ledger is what actually checks.
        taken, given = process_totals(name)
        if process.get("inputs") and taken != given:
            found.append(
                f"{where} does not balance: in {taken} g, out {given} g — a difference of "
                f"{taken - given} g. Every gram that leaves must arrive somewhere, and a loss "
                f"is declared as an output rather than subtracted"
            )
        for total, side in ((taken, "inputs"), (given, "outputs")):
            if total.denominator != 1:
                found.append(
                    f"{where} totals {total} g across its {side}, which is not a whole number "
                    f"of grams — a process measured in thirds of a gram is a process whose "
                    f"numbers are wrong"
                )
        for substance, quantity in process.get("outputs", ()):
            if substance not in substances:
                continue
            if _grams(substances[substance], quantity).denominator != 1:
                found.append(
                    f"{where} produces {quantity} of `{substance}`, which is not a whole "
                    f"number of grams — the conversion is exact or it is a defect"
                )
        for substance, quantity in process.get("inputs", ()):
            if substance not in substances:
                continue
            if _grams(substances[substance], quantity).denominator != 1:
                found.append(
                    f"{where} consumes {quantity} of `{substance}`, which is not a whole "
                    f"number of grams"
                )

        # A gather is not a creation: no inputs means the world already held it.
        if not process.get("inputs"):
            for substance, _quantity in process.get("outputs", ()):
                declared_source = substances.get(substance, {}).get("source")
                if declared_source == "made":
                    found.append(
                        f"{where} takes nothing in and produces `{substance}`, whose source is "
                        f"`made` — that is material from nothing, which is the one thing this "
                        f"table exists to refuse. A process with no inputs may only produce "
                        f"what the world already holds"
                    )

    # --- vocabularies, and the consumer rule ---------------------------------
    for kind, entries in declare.VOCABULARY.items():
        for entry in entries:
            if not any(
                process.get("requires", {}).get(kind) == entry
                for process in declare.PROCESSES.values()
            ):
                found.append(
                    f"{kind} `{entry}` is declared and no process requires it — the vocabulary "
                    f"is closed and an entry nothing uses is an intention wearing a name"
                )

    orphaned = set()
    for name, substance in substances.items():
        if name in consumed:
            continue
        if not substance.get("no_consumer"):
            found.append(
                f"substance `{name}` has no consumer and no declared reason. The schema's rule "
                f"is that a substance nothing uses is a defect — and a substance something will "
                f"use as soon as the rung above lands is unfinished work, which must say so "
                f"with `no_consumer`"
            )
        else:
            orphaned.add(name)
    return found


def gather_from_the_world() -> list:
    """Every process that draws on the world rather than on a stock, in grams.

    Not a defect and not a balance: a gather's other side is a deposit, so what it
    produces is exactly what the ground stops holding. Listed rather than skipped,
    because "source" is the one place in the design where mass enters the world.
    """
    drawn = []
    for name, process in declare.PROCESSES.items():
        if process.get("inputs"):
            continue
        for substance, quantity in process["outputs"]:
            drawn.append((name, substance, quantity_grams(substance, quantity)))
    return drawn


def open_items() -> list:
    """What the tables deliberately do not decide, printed as open every run.

    A substance nothing consumes **is** a finding worth reading — it says which rung
    is unfinished — so it is printed here rather than swallowed. Same posture as
    `design::verify`'s open list: what a green run does *not* mean.
    """
    items = []
    for name, substance in declare.SUBSTANCES.items():
        if substance.get("no_consumer"):
            items.append(
                f"`{name}` is declared and nothing consumes it: {substance['no_consumer']}"
            )
    for name, substance, grams in gather_from_the_world():
        items.append(f"`{name}` draws {grams} g of `{substance}` out of the world's own ground")
    items.append(
        "the ages: two tiers are declared and Q69 asked for as many as possible — the "
        "content is unbounded and an age is rows, so adding one touches no schema"
    )
    items.append(
        "the numbers in these tables are declared, not measured: every duration, labour "
        "cost, power figure and loss ratio is `[D]`, and the rung above the stone one is "
        "blocked on `[NS]` source rows rather than on this schema"
    )
    return items


def main() -> int:
    # The tables are UTF-8 and the reports use em dashes, which a Windows console
    # whose code page is not UTF-8 renders as mojibake -- and a defect report nobody
    # can read is the first step to a check nobody runs. Guarded, because a stream
    # that cannot be reconfigured is not a reason to fail the gate.
    try:
        import sys
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:  # pragma: no cover - console dependent
        pass
    found = defects()
    for item in found:
        print(f"schema: DEFECT: {item}")
    for item in open_items():
        print(f"schema: open — {item}")
    if found:
        print(f"schema: {len(found)} defect(s) in {len(declare.SUBSTANCES)} substances and "
              f"{len(declare.PROCESSES)} processes")
        return 1
    print(f"schema: {len(declare.SUBSTANCES)} substances, {len(declare.PROCESSES)} processes, "
          f"{len(declare.TIERS)} tiers, all balanced")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
