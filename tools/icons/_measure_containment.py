"""Throwaway: containment in the icon's frame vs in the mark's file frame.

Answers one question — was `tool-road`'s 0.3235 a trace fault or the check comparing
two different frames — by re-measuring the committed renders, so no re-render is
needed to know. Run, read, delete.

    blender --background --factory-startup --python tools/icons/_measure_containment.py
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import bpy

import generate
import shapes

REVIEW = generate.REVIEW
ICONS = (
    "tool-road", "tool-power", "tool-inspect", "tool-demolish",
    "vocab-settings", "ticket", "ledger", "help",
)


def read_rgba(path, px=96):
    with open(path, "rb") as handle:
        data = handle.read()
    if len(data) != px * px * 4:
        raise SystemExit(f"{path} is {len(data)} bytes, not {px}x{px}x4")
    return data


def grid_map(occupancy, render_cells, grid_side=12):
    """Where the mark is covered, where it is missed, and what else is drawn."""
    rows = []
    for y in range(grid_side):
        row = ""
        for x in range(grid_side):
            index = y * grid_side + x
            mark = occupancy[index] >= 0.5
            drawn = render_cells[index] >= 0.5
            row += "#" if mark and drawn else "M" if mark else "." if drawn else " "
        rows.append(row)
    return rows


def explain(icon, generation):
    entry = {"id": icon, "silhouette": shapes.silhouette_record(icon)}
    path = os.path.join(REVIEW, f"{icon}.{generation}.96.rgba")
    pixels = read_rgba(path)
    check = generate.silhouette_check(bpy, entry, pixels)
    mark = shapes.glyph_mask(bpy, entry["silhouette"]["glyph"])
    render_cells = generate.alpha_occupancy(pixels, 96)
    print(f"\n{icon}.{generation}  containment {check['containment']:.4f}")
    print("  # mark covered · M mark missed · . drawn and not declared")
    for row in grid_map(mark["occupancy"], render_cells):
        print("  " + row)
    print(f"  mark bodies {len(mark['bounds']['x']) and len(shapes.outline_for(bpy, entry['silhouette']['glyph'])['bodies'])}, "
          f"holes {len(shapes.outline_for(bpy, entry['silhouette']['glyph'])['holes'])}, "
          f"mark coverage {mark['coverage']}, pixels out of frame {mark['placed_outside_frame']}")
    # Alpha where the mark is, and where it is missed: a transparent material reads
    # as a miss even though something is drawn there.
    alphas = []
    for index, cell in enumerate(mark["occupancy"]):
        if cell < 0.5:
            continue
        cell_px = []
        x0, y0 = (index % 12) * 8, (index // 12) * 8
        for y in range(y0, y0 + 8):
            for x in range(x0, x0 + 8):
                cell_px.append(pixels[(y * 96 + x) * 4 + 3])
        alphas.append(sum(cell_px) / len(cell_px))
    if alphas:
        alphas.sort()
        print(f"  mean alpha over declared cells: min {alphas[0]:.0f}, "
              f"median {alphas[len(alphas) // 2]:.0f}, max {alphas[-1]:.0f} (threshold 127.5)")


def main():
    print()
    print(f"{'icon':16} {'generation':28} {'placed':>7} {'file':>7} {'agree':>7}")
    worst = {}
    for icon in ICONS:
        try:
            entry = {"id": icon, "silhouette": shapes.silhouette_record(icon)}
        except KeyError:
            print(f"{icon:16} {'—':28} {'no declared silhouette'}")
            continue
        for name in sorted(os.listdir(REVIEW)):
            if not name.startswith(icon + ".") or not name.endswith(".96.rgba"):
                continue
            generation = name[len(icon) + 1:-len(".96.rgba")]
            pixels = read_rgba(os.path.join(REVIEW, name))
            check = generate.silhouette_check(bpy, entry, pixels)
            if not check.get("judged"):
                print(f"{icon:16} {generation:28} {'unjudged':>7}  {check.get('why', '')}")
                continue
            placed = check["containment"]
            file_frame = check["containment_file_frame"]
            print(f"{icon:16} {generation:28} {placed:>7.4f} {file_frame:>7.4f} "
                  f"{check['occupancy_agreement']:>7.4f}")
            best = worst.get(icon)
            if best is None or placed > best:
                worst[icon] = placed
    print()
    print(f"floor {generate.SILHOUETTE_CONTAINMENT_FLOOR} (judged) — best candidate per icon:")
    for icon in ICONS:
        if icon in worst:
            mark = "PASS" if worst[icon] >= generate.SILHOUETTE_CONTAINMENT_FLOOR else "FAIL"
            print(f"  {icon:16} {worst[icon]:.4f}  {mark}")

    for icon, generation in (("tool-power", "a-md1-plate"), ("tool-power", "e-ios6-plate"),
                             ("tool-road", "d-alternate-turn")):
        explain(icon, generation)


main()
