"""Emit `src/materials/generated.rs` from the declaration.

The generated Rust source **is** the artifact. There is no second JSON and no
runtime parse: the game compiles the table in, so the "file is missing" case cannot
happen. What replaces it is the failure that *can* happen — a **stale** generated
file — and that is caught two ways:

* `python tools/materials/emit.py --check` exits non-zero when the file on disk
  differs from what would be written;
* the generated file carries `SOURCE_DIGEST`, an FNV-1a 64 over the bytes of the
  files it was derived from, and `src/materials/mod.rs` recomputes it from disk in a
  test. Mirrored in two languages on purpose: the check must be runnable by
  `cargo test`, which this project keeps free of Python and Blender.

    python tools/materials/emit.py          # write it
    python tools/materials/emit.py --check  # refuse if it is stale
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
ICONS = os.path.join(ROOT, "tools", "icons")
sys.path.insert(0, ICONS)
sys.path.insert(0, HERE)

import declare  # noqa: E402
import palette  # noqa: E402
import schema  # noqa: E402

OUT = os.path.join(ROOT, "src", "materials", "generated.rs")

#: The files this artifact is derived from, in a declared order — the digest is over
#: exactly these bytes, and both languages read the same list.
#:
#: `schema.py` is here although it contributes no *values*: it decides what may be
#: part of the tables, so a change to it changes what the generated file should
#: contain. A gate whose edits leave the artifact looking current is a gate that can
#: be loosened without anything noticing.
SOURCES = (
    os.path.join(HERE, "declare.py"),
    os.path.join(HERE, "schema.py"),
    os.path.join(ICONS, "palette.py"),
)


def fnv1a64(data: bytes) -> int:
    """FNV-1a, 64-bit. Trivial in both languages, which is why it is the digest."""
    digest = 0xCBF29CE484222325
    for byte in data:
        digest ^= byte
        digest = (digest * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return digest


def source_digest() -> int:
    digest = 0xCBF29CE484222325
    for path in SOURCES:
        with open(path, "rb") as handle:
            data = handle.read()
        for byte in data:
            digest ^= byte
            digest = (digest * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return digest


def entry_name(family: str, hue: str, level: str) -> str:
    return f"{family}:{hue}" if level == "body" else f"{family}:{hue}:{level}"


def resolve(family: str, hue: str, level: str):
    """One matrix entry: its OKLCH triple and its linear sRGB reading."""
    if family in declare.ICON_FAMILIES:
        triple = palette.material_value(family, hue, level)
    else:
        triple = palette.world_material_value(family, hue, level)
    return triple, palette.oklch(*triple)


def materials() -> list:
    out = []
    for family in list(declare.ICON_FAMILIES) + list(declare.WORLD_FAMILIES):
        for hue in palette.HUE_ANCHOR_NAMES:
            for level in declare.MATERIAL_LEVELS:
                triple, rgb = resolve(family, hue, level)
                out.append({
                    "name": entry_name(family, hue, level),
                    "family": family,
                    "hue": hue,
                    "level": level,
                    "l": triple[0], "c": triple[1], "h": triple[2],
                    "rgb": rgb,
                })
    return out


def token_colours(entries: list) -> tuple:
    """Every token the table owns, plus what moved and why.

    Four tokens are not literals at all: they **are** a surface's material read out
    (`SURFACE_TOKENS`), so the token cannot drift away from the material it claims
    to be. Where that changes the shipping value, the change is emitted as a
    divergence rather than applied quietly — the old value is still in the
    declaration, and the record keeps both.
    """
    by_name = {entry["name"]: entry for entry in entries}
    colours, divergences = [], []

    for name, value in sorted(declare.SHARED_TOKENS.items()):
        surface = declare.SURFACE_TOKENS.get(name)
        if surface is None:
            # A declared colour linear sRGB cannot hold is not the colour that was
            # declared. `hud.rs` used to clamp the offending channel to zero, which
            # is a silent chroma reduction (`Ink` shipped 0.034 of chroma it never
            # had); here the chroma is searched down until the colour fits and the
            # reduction is recorded.
            lightness, chroma, degrees = value
            resolved = palette._bounded_chroma(lightness, chroma, degrees)
            colours.append((name, palette.oklch(lightness, resolved, degrees),
                            (lightness, resolved, degrees)))
            if abs(resolved - chroma) > 1e-9:
                divergences.append((
                    name,
                    f"{name} declared oklch({lightness}, {chroma}, {degrees}), which linear "
                    f"sRGB cannot hold — the frame silently clipped it",
                    f"now oklch({lightness}, {round(resolved, 6)}, {degrees}), resolved in gamut",
                ))
            continue
        family, hue, level = declare.WORLD_SURFACES[surface][:3]
        if hue == "__anchor__":
            raise SystemExit(
                f"`{surface}` declares its hue as `__anchor__`; a token needs one "
                f"resolved value, so give that surface a concrete anchor"
            )
        entry = by_name[entry_name(family, hue, level)]
        colours.append((name, entry["rgb"], (entry["l"], entry["c"], entry["h"])))
        if tuple(round(x, 6) for x in value) != tuple(round(x, 6) for x in (entry["l"], entry["c"], entry["h"])):
            divergences.append((
                name,
                f"{name} was oklch({value[0]}, {value[1]}, {value[2]})",
                f"now the reading of {family}:{hue}:{level} — oklch({entry['l']}, {entry['c']}, {entry['h']})",
            ))

    for name, value in sorted(declare.ICON_COLOURS.items()):
        colours.append((name, palette.oklch(*value), value))

    return colours, divergences


def rust_string(text: str) -> str:
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def rust_rational(value) -> str:
    """A declared rational as two integers. **Never a float**: this is the type the
    ledger's exactness depends on, and emitting `0.6` for `26/10` would put the one
    value the schema refuses into the artifact that is supposed to enforce it.
    """
    from fractions import Fraction

    fraction = Fraction(value)
    return f"Rational {{ num: {fraction.numerator}, den: {fraction.denominator} }}"


def rust_quantities(pairs) -> str:
    return "&[" + ", ".join(
        f"({rust_string(name)}, {rust_rational(amount)})" for name, amount in pairs
    ) + "]"


def rust_f32(value) -> str:
    """The **shortest** decimal literal that reads back as the same `f32`.

    `repr()` gives the shortest string that round-trips a Python `float` — a 64-bit
    value — so almost every literal in this file carried digits an `f32` cannot
    hold, and clippy says so, correctly: writing a literal the compiler has to
    discard is the same defect class as a number with no home, because it looks like
    precision and is not.

    The shortest form is found against the interval the `f32` actually occupies: at
    each digit count, the rounded, floored and ceiled candidates are tried, and the
    first that maps back to the same `f32` wins. Rounding the *double* alone is not
    enough — the shortest literal is often not a rounding of it.
    """
    import struct
    from decimal import Decimal, ROUND_CEILING, ROUND_FLOOR, ROUND_HALF_EVEN, localcontext

    target = struct.pack("f", float(value))
    exact = struct.unpack("f", target)[0]
    if exact == 0.0:
        return "0.0"
    number = Decimal(exact)
    for digits in range(1, 10):
        step = Decimal(1).scaleb(number.adjusted() - digits + 1)
        for mode in (ROUND_HALF_EVEN, ROUND_FLOOR, ROUND_CEILING):
            with localcontext() as context:
                context.prec = digits + 2
                candidate = number.quantize(step, rounding=mode)
            if struct.pack("f", float(candidate)) != target:
                continue
            text = format(candidate, "f")
            if "." in text:
                text = text.rstrip("0").rstrip(".")
            if not text or text == "-0":
                text = "0"
            return text + ".0" if "." not in text else text
    return repr(exact)


def rust_rgb(rgb) -> str:
    return "[" + ", ".join(rust_f32(channel) for channel in rgb) + "]"


def rust_rgba(rgb) -> str:
    return "[" + ", ".join(rust_f32(channel) for channel in rgb) + ", 1.0]"


def render() -> str:
    entries = materials()
    colours, divergences = token_colours(entries)
    digest = source_digest()
    out = []
    add = out.append

    add("//! GENERATED FILE — derive it with `python tools/materials/emit.py`; never hand-edit it.")
    add("//!")
    add("//! One home for the material truth: the icon pipeline, this crate's renderer and")
    add("//! the world's sim all read these values, and `tests` in `mod.rs` refuses a build")
    add("//! whose generated source no longer matches the declaration it came from.")
    add("")
    add(f"pub const SOURCE_DIGEST: u64 = 0x{digest:016X};")
    add('pub const GENERATED_BY: &str = "tools/materials/emit.py";')
    add("")
    add("/// One declared material family: its own lightness, the chroma a hue variation")
    add("/// may carry, and the chroma and hue its untinted body carries.")
    add("#[derive(Clone, Copy, Debug)]")
    add("pub struct Family {")
    add("    pub name: &'static str,")
    add("    pub lightness: f32,")
    add("    pub variant_chroma: f32,")
    add("    pub natural_chroma: f32,")
    add("    pub natural_hue: f32,")
    add("    pub note: &'static str,")
    add("}")
    add("")
    add("/// The icon set's families: exactly seven. A family the world needs does not")
    add("/// appear here, so an icon cannot be made of one.")
    add("pub const ICON_FAMILIES: &[Family] = &[")
    for name, (lightness, variant, natural_c, natural_h, note) in declare.ICON_FAMILIES.items():
        add(f"    Family {{ name: {rust_string(name)}, lightness: {rust_f32(lightness)}, "
            f"variant_chroma: {rust_f32(variant)}, natural_chroma: {rust_f32(natural_c)}, "
            f"natural_hue: {rust_f32(natural_h)}, note: {rust_string(note)} }},")
    add("];")
    add("")
    add("/// The world's own families, resolvable and never available to an icon.")
    add("pub const WORLD_FAMILIES: &[Family] = &[")
    for name, (lightness, variant, natural_c, natural_h, note) in declare.WORLD_FAMILIES.items():
        add(f"    Family {{ name: {rust_string(name)}, lightness: {rust_f32(lightness)}, "
            f"variant_chroma: {rust_f32(variant)}, natural_chroma: {rust_f32(natural_c)}, "
            f"natural_hue: {rust_f32(natural_h)}, note: {rust_string(note)} }},")
    add("];")
    add("")
    add("/// How colour physically arrives, per family, and the most chroma that mechanism")
    add("/// can carry. Asking for more is asking for a different material.")
    add("pub const MECHANISMS: &[(&str, &str, f32)] = &[")
    for name, (mechanism, ceiling) in declare.FAMILY_COLOUR.items():
        add(f"    ({rust_string(name)}, {rust_string(mechanism)}, {rust_f32(ceiling)}),")
    add("];")
    add("")
    add("/// Declared lightness offsets on a family's own lightness — one material read")
    add("/// twice, rather than two materials.")
    add("pub const LEVELS: &[(&str, f32)] = &[")
    for name, offset in declare.MATERIAL_LEVELS.items():
        add(f"    ({rust_string(name)}, {rust_f32(offset)}),")
    add("];")
    add(f"pub const LIGHTNESS_BOUNDS: (f32, f32) = ({rust_f32(declare.MATERIAL_LIGHTNESS_BOUNDS[0])}, "
        f"{rust_f32(declare.MATERIAL_LIGHTNESS_BOUNDS[1])});")
    add("")
    add("/// The hue ring, and what backs each anchor. A slot says so rather than")
    add("/// borrowing a measurement's authority.")
    add("pub const ANCHORS: &[(&str, f32, &str)] = &[")
    for name, degrees, backing in declare.HUE_ANCHORS:
        add(f"    ({rust_string(name)}, {rust_f32(degrees)}, {rust_string(backing)}),")
    add("];")
    add('pub const NATURAL: &str = "natural";')
    add("")
    add("/// One resolved entry: family x anchor x level, in both vocabularies — OKLCH,")
    add("/// which is what a designer reads, and linear sRGB, which is what the frame takes.")
    add("#[derive(Clone, Copy, Debug)]")
    add("pub struct Material {")
    add("    pub name: &'static str,")
    add("    pub family: &'static str,")
    add("    pub hue: &'static str,")
    add("    pub level: &'static str,")
    add("    pub lightness: f32,")
    add("    pub chroma: f32,")
    add("    pub hue_degrees: f32,")
    add("    pub rgb: [f32; 3],")
    add("}")
    add("")
    add(f"pub const MATERIALS: &[Material] = &[")
    for entry in entries:
        add(f"    Material {{ name: {rust_string(entry['name'])}, family: {rust_string(entry['family'])}, "
            f"hue: {rust_string(entry['hue'])}, level: {rust_string(entry['level'])}, "
            f"lightness: {rust_f32(entry['l'])}, chroma: {rust_f32(entry['c'])}, "
            f"hue_degrees: {rust_f32(entry['h'])}, rgb: {rust_rgb(entry['rgb'])} }},")
    add("];")
    add("")
    add("/// The colours this table owns. `hud.rs` resolves these rather than carrying")
    add("/// literals, so an icon is never measured against a copy of a colour.")
    add("pub const TOKEN_COLOURS: &[(&str, [f32; 4])] = &[")
    for name, rgb, _triple in colours:
        add(f"    ({rust_string(name)}, {rust_rgba(rgb)}),")
    add("];")
    add("")
    add("/// Tokens that **are** a surface's material read out, so the two cannot drift.")
    add("pub const SURFACE_TOKENS: &[(&str, &str)] = &[")
    for token, surface in declare.SURFACE_TOKENS.items():
        add(f"    ({rust_string(token)}, {rust_string(surface)}),")
    add("];")
    add("")
    add("/// Where the table's reading of a surface differs from the literal it replaced.")
    add("/// Superseded, never erased: the shipping value is still in the declaration.")
    add("pub const DIVERGENCES: &[(&str, &str, &str)] = &[")
    for name, was, now in divergences:
        add(f"    ({rust_string(name)}, {rust_string(was)}, {rust_string(now)}),")
    add("];")
    add("")
    add("/// part, family, hue, level, note — every part of the world, as a claim.")
    add("pub const WORLD_SURFACES: &[(&str, &str, &str, &str, &str)] = &[")
    for part, (family, hue, level, note) in declare.WORLD_SURFACES.items():
        add(f"    ({rust_string(part)}, {rust_string(family)}, {rust_string(hue)}, "
            f"{rust_string(level)}, {rust_string(note)}),")
    add("];")
    add("")
    add("/// Surfaces the world does not have yet: a declared absence, never an invention.")
    add("pub const DEFERRED_SURFACES: &[(&str, &str)] = &[")
    for name, why in declare.DEFERRED_SURFACES.items():
        add(f"    ({rust_string(name)}, {rust_string(why)}),")
    add("];")
    add("")

    def number_table(name: str, table: dict, doc: str, kind: str = "f32"):
        add(f"/// {doc}")
        add(f"pub const {name}: &[(&str, {kind})] = &[")
        for key, value in table.items():
            rendered = rust_string(value) if isinstance(value, str) else rust_f32(value)
            add(f"    ({rust_string(key)}, {rendered}),")
        add("];")
        add("")

    number_table("PART_PRICE", declare.PART_PRICE,
                 "A structure's build cost is the sum of its parts' prices, per a173.")
    number_table("UPKEEP_PER_MONTH", declare.UPKEEP_PER_MONTH, "Credits per month, per part, by family.")
    # Wear, floor and ceiling are **rationals** rather than floats: weathering moves mass
    # (Q109/Q114) and mass here is integer grams, so the rate is grams-per-gram and the
    # thresholds are shares of a structure's own mass. `REPAIR_SHARE` stays a float because
    # it prices an act in credits, and credits are not mass.
    add("/// Wear per sim-day, by family: grams lost per gram held, as an exact rational.")
    add("pub const DECAY_PER_DAY: &[(&str, i64, i64)] = &[")
    for family, rate in declare.DECAY_PER_DAY.items():
        add(f"    ({rust_string(family)}, {rate.numerator}, {rate.denominator}),")
    add("];")
    add("")
    add("/// Below this share of its own mass a structure owes a repair.")
    add(f"pub const REPAIR_FLOOR: (i64, i64) = "
        f"({declare.REPAIR_FLOOR.numerator}, {declare.REPAIR_FLOOR.denominator});")
    add("/// Above this share it is fully maintained and nothing is owed.")
    add(f"pub const REPAIR_CEILING: (i64, i64) = "
        f"({declare.REPAIR_CEILING.numerator}, {declare.REPAIR_CEILING.denominator});")
    add(f"pub const REPAIR_SHARE: f32 = {rust_f32(declare.REPAIR_SHARE)};")
    add("")
    number_table("CONDUCTION", declare.CONDUCTION, "Whether a family carries power, and how much.")
    number_table("SURFACE_SPEED", declare.SURFACE_SPEED, "Travel speed over a surface; 0.0 means unbuildable.")
    number_table("DESIRABILITY", declare.DESIRABILITY, "How a structure moves demand around it.")
    number_table("NUISANCE", declare.NUISANCE, "Noise and pollution per structure, sampled at read points.")
    # --- the substance and process tables (C9 phase 3) ----------------------
    add("/// One declared rational: a numerator over a denominator, in integers.")
    add("///")
    add("/// A float is what the schema refuses, because a ledger in mixed units balances")
    add("/// only if every conversion is exact — so the type holding a density, a unit mass,")
    add("/// a rot rate and a quantity has no float in it at all.")
    add("#[derive(Clone, Copy, Debug, PartialEq, Eq)]")
    add("pub struct Rational {")
    add("    pub num: i64,")
    add("    pub den: i64,")
    add("}")
    add("")
    add("/// One substance: what it is measured in, what it presents as, and where it")
    add("/// comes from.")
    add("#[derive(Clone, Copy, Debug)]")
    add("pub struct Substance {")
    add("    pub name: &'static str,")
    add("    pub family: &'static str,")
    add("    pub hue: &'static str,")
    add("    /// Mass | Volume | Count | Gas — the canonical unit the ledger counts in.")
    add("    pub unit: &'static str,")
    add("    /// g/mL, required for Volume and Gas; zero where the unit does not need it.")
    add("    pub density: Rational,")
    add("    /// Grams per unit, required for Count; zero where the unit does not need it.")
    add("    pub unit_mass: Rational,")
    add("    /// Condition lost per sim-day. All zero today: rot's mechanism arrives with")
    add("    /// phase 7's couplings, and a rate nothing reads would be a placeholder.")
    add("    pub rot_per_day: Rational,")
    add("    pub tags: &'static [&'static str],")
    add("    pub source: &'static str,")
    add("    /// The natural unit a person reads it in, and its exact grams per unit. Never")
    add("    /// used in arithmetic — Q67's per-substance units are a reading, not a conversion.")
    add("    pub display_unit: &'static str,")
    add("    pub display_grams: Rational,")
    add("    /// Why nothing consumes it, or empty when something does.")
    add("    pub no_consumer: &'static str,")
    add("    /// Where it goes when a process gives it up: `holding` means the site it was made")
    add("    /// at, `atmosphere` means the open air. A gas must declare one: a gas that lands in")
    add("    /// a site holding is a gas somebody is holding, which is a contradiction in terms.")
    add("    pub sink: &'static str,")
    add("    pub note: &'static str,")
    add("}")
    add("")
    add("/// What a process costs to run, as numbers rather than as prose (Q37's rule).")
    add("#[derive(Clone, Copy, Debug)]")
    add("pub struct Mechanism {")
    add("    pub heat_c: i64,")
    add("    /// `material` or `flame`, empty when nothing is heated. The two readings of")
    add("    /// \"kiln temperature\" differ by ~600 °C and both are correct.")
    add("    pub heat_kind: &'static str,")
    add("    pub hours: Rational,")
    add("    pub labour_hours: Rational,")
    add("    pub power_kw: i64,")
    add("}")
    add("")
    add("/// One process: what it takes, what it gives, and the number behind it.")
    add("#[derive(Clone, Copy, Debug)]")
    add("pub struct Process {")
    add("    pub name: &'static str,")
    add("    pub tier: &'static str,")
    add("    pub inputs: &'static [(&'static str, Rational)],")
    add("    pub outputs: &'static [(&'static str, Rational)],")
    add("    pub mechanism: Mechanism,")
    add("    /// What must exist for the work to be possible: (kind, name) pairs, each a")
    add("    /// declared entry in the vocabulary that kind owns.")
    add("    pub requires: &'static [(&'static str, &'static str)],")
    add("    pub note: &'static str,")
    add("}")
    add("")
    add("/// The ages, as rows. An age is *reached* rather than assumed, so the ordinal is")
    add("/// what a progression compares against.")
    add("pub const TIERS: &[(&str, i64, &str)] = &[")
    for name, ordinal, note in declare.TIERS:
        add(f"    ({rust_string(name)}, {ordinal}, {rust_string(note)}),")
    add("];")
    add("")
    add("/// Every declared substance. The type has no float in it, which is the point.")
    add("pub const SUBSTANCES: &[Substance] = &[")
    for name, s in declare.SUBSTANCES.items():
        tags = ", ".join(rust_string(tag) for tag in s.get("tags", ()))
        display_unit, display_grams = s.get("display", ("", None))
        add("    Substance {")
        add(f"        name: {rust_string(name)},")
        add(f"        family: {rust_string(s['family'])}, hue: {rust_string(s['hue'])}, "
            f"unit: {rust_string(s['unit'])},")
        add(f"        density: {rust_rational(s.get('density', 0))}, "
            f"unit_mass: {rust_rational(s.get('unit_mass', 0))}, "
            f"rot_per_day: {rust_rational(s.get('rot', 0))},")
        add(f"        tags: &[{tags}] as &[&str],")
        add(f"        source: {rust_string(s['source'])}, "
            f"display_unit: {rust_string(display_unit)}, "
            f"display_grams: {rust_rational(display_grams or 0)},")
        add(f"        no_consumer: {rust_string(s.get('no_consumer', ''))}, "
            f"sink: {rust_string(s.get('sink', 'holding'))}, "
            f"note: {rust_string(s.get('note', ''))},")
        add("    },")
    add("];")
    add("")
    add("/// Every declared process. `SCHEMA.md` is the contract these rows must satisfy,")
    add("/// and `schema.py` is the gate that refuses a row which does not.")
    add("pub const PROCESSES: &[Process] = &[")
    for name, p in declare.PROCESSES.items():
        mechanism = p["mechanism"]
        requires = ", ".join(
            f"({rust_string(kind)}, {rust_string(entry)})"
            for kind, entry in sorted(p.get("requires", {}).items())
        )
        add("    Process {")
        add(f"        name: {rust_string(name)}, tier: {rust_string(p['tier'])},")
        add(f"        inputs: {rust_quantities(p.get('inputs', ()))},")
        add(f"        outputs: {rust_quantities(p.get('outputs', ()))},")
        add(f"        mechanism: Mechanism {{ heat_c: {int(mechanism.get('heat_c', 0))}, "
            f"heat_kind: {rust_string(mechanism.get('heat_kind', ''))}, "
            f"hours: {rust_rational(mechanism.get('hours', 0))}, "
            f"labour_hours: {rust_rational(mechanism.get('labour_hours', 0))}, "
            f"power_kw: {int(mechanism.get('power_kw', 0))} }},")
        add(f"        requires: &[{requires}] as &[(&str, &str)],")
        add(f"        note: {rust_string(p.get('note', ''))},")
        add("    },")
    add("];")
    add("")
    add("/// The closed vocabularies a process may require from: what exists, as rows.")
    add("/// Empty today because the first rung is hand work — and an entry no process")
    add("/// requires is refused by the gate, so this cannot fill with intentions.")
    for kind in ("structure", "tool", "skill"):
        constant = f"{kind.upper()}S"
        add(f"pub const {constant}: &[(&str, &str)] = &[")
        for name, note in declare.VOCABULARY[kind].items():
            add(f"    ({rust_string(name)}, {rust_string(note)}),")
        add("];")
        add("")
    add("/// What the substance and process tables deliberately do not decide.")
    add("pub const SCHEMA_OPEN: &[&str] = &[")
    for item in schema.open_items():
        add(f"    {rust_string(item)},")
    add("];")
    add("")
    add("/// What this table cannot check, printed every run rather than implied away.")
    add("pub const OPEN: &[&str] = &[")
    for item in declare.OPEN:
        add(f"    {rust_string(item)},")
    add("];")
    add("")
    return "\n".join(out) + "\n"


def main() -> int:
    # The gate runs before anything is written, and it **fails closed**: a table with a
    # defect does not become an artifact. Every defect is printed rather than the
    # first, for the same reason `design::verify` reports all of them — a check that
    # reports one problem per run is a check somebody stops running.
    found = schema.defects()
    for item in found:
        print(f"schema: DEFECT: {item}")
    if found:
        print(f"materials: refusing to emit — {len(found)} defect(s) in the substance and "
              f"process tables")
        return 1
    text = render()
    check = "--check" in sys.argv
    existing = None
    if os.path.exists(OUT):
        with open(OUT, "r", encoding="utf-8", newline="") as handle:
            existing = handle.read()
    if check:
        if existing == text:
            print(f"materials: {OUT} is current ({len(text.splitlines())} lines)")
            return 0
        print(f"materials: {OUT} is STALE — re-run `python tools/materials/emit.py`")
        return 1
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)
    entries = materials()
    colours, divergences = token_colours(entries)
    print(f"materials: wrote {len(entries)} entries, {len(colours)} tokens, "
          f"{len(declare.WORLD_SURFACES)} world parts, digest 0x{source_digest():016X}")
    for name, was, now in divergences:
        print(f"materials: divergence — {was}; {now}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
