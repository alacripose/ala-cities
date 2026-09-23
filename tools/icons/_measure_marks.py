"""Read the reference marks the six families will be read off (C8 a192).

Owed by a192: the bin's vocabulary — and, since five of the six marks depict the
object their family is for, the other four can be read the same way. This is the
evidence generator, not a gate: it prints what each mark measures so a declared
endpoint can point at a number rather than at a memory.

Run it the way the pipeline runs anything that reads a PNG, because nothing in this
tree has a PNG decoder and `bpy.data.images` is the only reader there is:

    blender --background --factory-startup --python tools/icons/_measure_marks.py
"""

import math
import os
import sys

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import shapes  # noqa: E402

#: The six reviewed icons and the mark each declares. `tool-power`'s mark is the one
#: entry here whose object is *not* what its retired recipe built: the mark is a
#: power symbol (a ring with a gap and a stem) while the recipe was a pylon.
MARKS = (
    ("vocab-settings", "action/settings"),
    ("tool-road", "maps/add_road"),
    ("tool-power", "action/power_settings_new"),
    ("tool-inspect", "action/search"),
    ("tool-demolish", "action/delete"),
    ("ticket", "notification/confirmation_number"),
)


def mask_of(glyph):
    """The mark as a top-down boolean grid, plus its size."""
    width, height, pixels = shapes._load_glyph(bpy, glyph)
    # Blender hands back floats, and the pack's marks are flat black on transparency,
    # so the alpha channel *is* the silhouette. 0.5 is the threshold every other
    # reader in this pipeline uses, so the numbers here are comparable with theirs.
    grid = [[pixels[((height - 1 - y) * width + x) * 4 + 3] >= 0.5
             for x in range(width)]
            for y in range(height)]
    return width, height, grid


def components(grid, want_ink, width, height):
    """8-connected components of ink, or of background, with their areas."""
    seen = [[False] * width for _ in range(height)]
    found = []
    for y0 in range(height):
        for x0 in range(width):
            if seen[y0][x0] or grid[y0][x0] != want_ink:
                continue
            stack = [(x0, y0)]
            seen[y0][x0] = True
            area = 0
            while stack:
                x, y = stack.pop()
                area += 1
                for dx in (-1, 0, 1):
                    for dy in (-1, 0, 1):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < width and 0 <= ny < height \
                                and not seen[ny][nx] and grid[ny][nx] == want_ink:
                            seen[ny][nx] = True
                            stack.append((nx, ny))
            found.append(area)
    return sorted(found, reverse=True)


def voids(grid, width, height):
    """Enclosed background: components of background not reachable from the border."""
    seen = [[False] * width for _ in range(height)]
    stack = []
    for x in range(width):
        for y in (0, height - 1):
            if not grid[y][x] and not seen[y][x]:
                seen[y][x] = True
                stack.append((x, y))
    for y in range(height):
        for x in (0, width - 1):
            if not grid[y][x] and not seen[y][x]:
                seen[y][x] = True
                stack.append((x, y))
    while stack:
        x, y = stack.pop()
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                nx, ny = x + dx, y + dy
                if 0 <= nx < width and 0 <= ny < height \
                        and not seen[ny][nx] and not grid[ny][nx]:
                    seen[ny][nx] = True
                    stack.append((nx, ny))
    areas = []
    for y0 in range(height):
        for x0 in range(width):
            if grid[y0][x0] or seen[y0][x0]:
                continue
            stack = [(x0, y0)]
            seen[y0][x0] = True
            area = 0
            while stack:
                x, y = stack.pop()
                area += 1
                for dx in (-1, 0, 1):
                    for dy in (-1, 0, 1):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < width and 0 <= ny < height \
                                and not seen[ny][nx] and not grid[ny][nx]:
                            seen[ny][nx] = True
                            stack.append((nx, ny))
            areas.append(area)
    return sorted(areas, reverse=True)


def radial_features(grid, centre, radius, rings=(0.45, 0.62, 0.80, 0.92)):
    """Runs of ink around a ring, at several fractions of the mark's outer radius.

    A run is one radial feature crossing that ring, so this is read as a count:
    a gear's teeth at 0.92, a hub's spokes at 0.62, a bore at 0.45.
    """
    cx, cy = centre
    out = {}
    for fraction in rings:
        r = radius * fraction
        samples = []
        for step in range(720):
            angle = step * 2.0 * math.pi / 720.0
            x = int(round(cx + math.cos(angle) * r))
            y = int(round(cy + math.sin(angle) * r))
            inside = 0 <= x < len(grid[0]) and 0 <= y < len(grid)
            samples.append(bool(grid[y][x]) if inside else False)
        runs = 0
        for index, value in enumerate(samples):
            if value and not samples[index - 1]:
                runs += 1
        out[fraction] = runs
    return out


def ink_extent(grid, width, height):
    """Bounding box, coverage and the centroid, in pixels, top-down."""
    xs, ys, count = [], [], 0
    for y in range(height):
        for x in range(width):
            if grid[y][x]:
                xs.append(x)
                ys.append(y)
                count += 1
    box = (min(xs), min(ys), max(xs), max(ys))
    centre = (sum(xs) / count, sum(ys) / count)
    return box, count, centre


def row_widths(grid, width, height):
    """Per-row ink width, left to right: a bin's lid is the widest band."""
    rows = []
    for y in range(height):
        x_in = [x for x in range(width) if grid[y][x]]
        rows.append((min(x_in), max(x_in), len(x_in)) if x_in else None)
    return rows


def runs_along(grid, x, height):
    """Ink runs down one column: a road's dashes are runs along its length."""
    column = [grid[y][x] for y in range(height)]
    runs, ink = 0, 0
    for index, value in enumerate(column):
        if value:
            ink += 1
            if not column[index - 1]:
                runs += 1
    return runs, ink


def report(icon_id, glyph):
    width, height, grid = mask_of(glyph)
    box, count, centre = ink_extent(grid, width, height)
    span = max(box[2] - box[0] + 1, box[3] - box[1] + 1)
    radius = span / 2.0
    pieces = components(grid, True, width, height)
    holes = voids(grid, width, height)
    print(f"\n=== {icon_id}  ({glyph})")
    print(f"    mark: {width}x{height} px, ink {count} px ({100 * count / (width * height):.1f}%), "
          f"box {box[2] - box[0] + 1}x{box[3] - box[1] + 1}, aspect "
          f"{(box[2] - box[0] + 1) / (box[3] - box[1] + 1):.2f}")
    print(f"    pieces {len(pieces)} {pieces[:6]}, voids {len(holes)} {holes[:6]}")
    print(f"    radial runs at 0.45/0.62/0.80/0.92 of R: "
          f"{list(radial_features(grid, centre, radius).values())}")
    rows = row_widths(grid, width, height)
    widths = [row[1] - row[0] + 1 for row in rows if row]
    if widths:
        widest = max(widths)
        body = sorted(widths)[len(widths) // 4]
        print(f"    row width: widest {widest} px, lower-quartile {body} px, "
              f"taper {widths[-1] / widest:.2f} (bottom/widest)")
    # The ribbon case: the column with the most ink, read down its length.
    best = max(range(width), key=lambda x: sum(1 for y in range(height) if grid[y][x]))
    runs, ink = runs_along(grid, best, height)
    print(f"    centre column {best}: {runs} runs, {ink} px of ink "
          f"({100 * ink / max(1, box[3] - box[1] + 1):.0f}% of its height)")


def main():
    for icon_id, glyph in MARKS:
        report(icon_id, glyph)
    print("\nRead as evidence for the declared endpoints (a188), not as a gate.")


main()
