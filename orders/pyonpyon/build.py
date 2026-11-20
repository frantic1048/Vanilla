#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["fonttools==4.67.0"]
# ///
"""Build Pyonpyon.otf, a small symbol font, from the SVGs in glyphs/.

SVG glyphs have a 1000x1000 viewBox and a single <path>; holes are contours
drawn in the opposite direction (nonzero winding). Composite glyphs (such as
the prompt badge pieces) are drawn in code. Square glyphs (advance 1000), such
as tippy, show about two cells wide in terminals that let them overflow into a
following space (WezTerm's default); the badge pieces are exactly one cell.

Code points come from Supplementary Private Use Area-B (U+100000 onwards),
clear of the Nerd Fonts ranges in the BMP Private Use Area and Plane 15.

Run from anywhere: `uv run --script orders/pyonpyon/build.py`. README.md lists
the glyphs; add a row there when appending to GLYPHS.
"""

import re
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.misc.timeTools import timestampFromString
from fontTools.pens.areaPen import AreaPen
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.reverseContourPen import ReverseContourPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.pens.transformPen import TransformPen
from fontTools.svgLib.path import parse_path

HERE = Path(__file__).resolve().parent
FAMILY = "Pyonpyon"
UPM = 1000
ASCENT, DESCENT = 800, -200
# SVG y grows downwards; place the SVG's y=860 on the baseline-relative origin
# so a glyph drawn in the lower part of the viewBox sits on the baseline.
BASELINE_IN_SVG = 860

# Flip y and move the baseline: (x, y) -> (x, BASELINE_IN_SVG - y).
SVG_TO_FONT = (1, 0, 0, -1, 0, BASELINE_IN_SVG)

# Fantasque Sans Mono's cell: 1060/2048 em wide. Glyphs meant to sit side by
# side in single cells use this advance so they butt up exactly.
CELL = 518

# The prompt badge: <pill-left><tippy-inverse-left><tippy-inverse-right><pill-right>,
# all one foreground color, sharing BADGE_BOTTOM..BADGE_TOP. Tippy is cut out of
# the two middle cells (split down its middle) at INVERSE_TIPPY_SCALE; its eyes
# and mouth come back as ink. Pieces overlap their neighbours by OVERLAP so no
# hairline shows where they meet.
# The badge fills the whole cell height (Fantasque's descent to ascent plus line
# gap, about -200..855), slightly overshooting, so a cell background on the two
# middle cells only shows through Tippy's cut-out.
BADGE_BOTTOM, BADGE_TOP = -205, 860
# Pill ends are half-ellipses: as tall as the badge, PILL_END_WIDTH wide.
PILL_END_WIDTH = 470
INVERSE_TIPPY_SCALE = 1.0
OVERLAP = 6
TIPPY_CENTER_Y = 272.5  # vertical center of Tippy's bounds in font units

# Left half of Tippy (tippy.svg coordinates), closed along its center line,
# pushed OVERLAP past the middle; plus the left eye and the left half of the
# mouth. The right half is the mirror image.
TIPPY_LEFT_HALF = """
    M 130 430
    C 128 360, 150 290, 195 250
    C 205 238, 222 236, 236 246
    C 266 268, 300 288, 340 296
    C 390 286, 445 281, 500 281
    L {mid} 281 L {mid} 935 L 500 935
    C 280 935, 125 885, 95 765
    C 65 650, 85 520, 130 430
    Z
"""
TIPPY_LEFT_EYE = "M 350 570 a 48 53 0 1 0 96 0 a 48 53 0 1 0 -96 0 Z"
TIPPY_LEFT_MOUTH = "M 482 640 L 482 680 L 446 716 L 470 738 L 500 708 L {mid} {mouth_y} L {mid} 640 Z"


def path_data(svg: Path) -> str:
    match = re.search(r'\bd="([^"]+)"', svg.read_text())
    if not match:
        raise SystemExit(f"{svg}: no <path d=...> found")
    return match.group(1)


def draw_svg(pen, svg: Path) -> None:
    parse_path(path_data(svg), TransformPen(pen, SVG_TO_FONT))


def draw_tippy(pen) -> None:
    draw_svg(pen, HERE / "glyphs" / "tippy.svg")


def contour(draw) -> RecordingPen:
    rec = RecordingPen()
    draw(rec)
    return rec


def area(rec: RecordingPen) -> float:
    pen = AreaPen()
    rec.replay(pen)
    return pen.value


def emit(pen, rec: RecordingPen, ink: bool) -> None:
    """Replay a contour so ink runs one way and holes the other (nonzero)."""
    if (area(rec) > 0) != ink:
        rec.replay(ReverseContourPen(pen))
    else:
        rec.replay(pen)


def tippy_half_transform():
    """tippy.svg coordinates to the left-half glyph: Tippy's middle on the cell edge."""
    s = INVERSE_TIPPY_SCALE
    center_y = (BADGE_BOTTOM + BADGE_TOP) / 2
    # x: s * (x - 500) + CELL; y: s * ((BASELINE_IN_SVG - y) - TIPPY_CENTER_Y) + center_y
    return (s, 0, 0, -s, CELL - 500 * s, s * (BASELINE_IN_SVG - TIPPY_CENTER_Y) + center_y)


def draw_inverse_left(pen, mirror: bool = False) -> None:
    mid = 500 + OVERLAP / INVERSE_TIPPY_SCALE
    t = tippy_half_transform()
    if mirror:
        # Right half: mirror x within the cell, x -> CELL - x.
        a, b, c, d, e, f = t
        t = (-a, b, -c, d, CELL - e, f)
    left, right = -OVERLAP, CELL + OVERLAP
    rect = contour(lambda p: (
        p.moveTo((left, BADGE_BOTTOM)), p.lineTo((right, BADGE_BOTTOM)),
        p.lineTo((right, BADGE_TOP)), p.lineTo((left, BADGE_TOP)), p.closePath(),
    ))
    notch = contour(lambda p: parse_path(TIPPY_LEFT_HALF.format(mid=mid), TransformPen(p, t)))
    eye = contour(lambda p: parse_path(TIPPY_LEFT_EYE, TransformPen(p, t)))
    mouth = contour(lambda p: parse_path(
        TIPPY_LEFT_MOUTH.format(mid=mid, mouth_y=708 + (mid - 500)), TransformPen(p, t)))
    emit(pen, rect, ink=True)
    emit(pen, notch, ink=False)
    emit(pen, eye, ink=True)
    emit(pen, mouth, ink=True)


def draw_inverse_right(pen) -> None:
    draw_inverse_left(pen, mirror=True)


def draw_pill_end(pen, left_end: bool) -> None:
    """Half ellipse in one cell: round on the outside, flat (and overlapping)
    against the inverse Tippy."""
    rx, ry = PILL_END_WIDTH, (BADGE_TOP - BADGE_BOTTOM) / 2
    kx, ky = 0.5523 * rx, 0.5523 * ry  # cubic approximation of a quarter ellipse
    cy = BADGE_BOTTOM + ry
    if left_end:
        flat, cx, sign = CELL + OVERLAP, CELL, -1
    else:
        flat, cx, sign = -OVERLAP, 0, 1
    edge = cx + sign * rx
    rec = contour(lambda p: (
        p.moveTo((flat, BADGE_BOTTOM)),
        p.lineTo((cx, BADGE_BOTTOM)),
        p.curveTo((cx + sign * kx, BADGE_BOTTOM), (edge, cy - ky), (edge, cy)),
        p.curveTo((edge, cy + ky), (cx + sign * kx, BADGE_TOP), (cx, BADGE_TOP)),
        p.lineTo((flat, BADGE_TOP)),
        p.closePath(),
    ))
    emit(pen, rec, ink=True)


# (glyph name, code point, advance, drawing function). Append new glyphs; never
# renumber once released.
GLYPHS = [
    ("tippy", 0x100000, UPM, draw_tippy),
    ("tippy-inverse-left", 0x100001, CELL, draw_inverse_left),
    ("tippy-inverse-right", 0x100002, CELL, draw_inverse_right),
    ("pill-left", 0x100003, CELL, lambda pen: draw_pill_end(pen, left_end=True)),
    ("pill-right", 0x100004, CELL, lambda pen: draw_pill_end(pen, left_end=False)),
]


def charstring(draw):
    pen = T2CharStringPen(UPM, None)
    draw(pen)
    return pen.getCharString()


def left_side_bearing(charstr) -> int:
    bounds = BoundsPen(None)
    charstr.draw(bounds)
    return round(bounds.bounds[0]) if bounds.bounds else 0


def main() -> None:
    notdef = T2CharStringPen(UPM // 2, None).getCharString()
    charstrings = {".notdef": notdef}
    for name, _, _, draw in GLYPHS:
        charstrings[name] = charstring(draw)

    fb = FontBuilder(UPM, isTTF=False)
    fb.setupGlyphOrder(list(charstrings))
    fb.setupCharacterMap({cp: name for name, cp, _, _ in GLYPHS})
    fb.setupCFF(f"{FAMILY}-Regular", {"FullName": f"{FAMILY} Regular"}, charstrings, {})
    metrics = {".notdef": (UPM // 2, 0)}
    for name, _, advance, _ in GLYPHS:
        charstrings[name].compile()
        metrics[name] = (advance, left_side_bearing(charstrings[name]))
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=ASCENT, descent=DESCENT)
    fb.setupNameTable({"familyName": FAMILY, "styleName": "Regular"})
    fb.setupOS2(
        sTypoAscender=ASCENT, sTypoDescender=DESCENT, sTypoLineGap=0,
        usWinAscent=ASCENT, usWinDescent=-DESCENT,
    )
    fb.setupPost()
    # Fixed dates keep rebuilds byte-identical, so git only sees real changes.
    fb.font["head"].created = fb.font["head"].modified = timestampFromString("Fri Nov 20 00:00:00 2026")
    fb.font.recalcTimestamp = False
    out = HERE / f"{FAMILY}.otf"
    fb.save(out)
    print(f"wrote {out} ({len(GLYPHS)} glyphs)")
    write_previews(fb.font)


# Previews for the README, drawn from the built font's outlines.
PREVIEW_INK = "#62d0f0"     # Chino's aqua, as in the prompt badge
PREVIEW_TILE = "#142050"    # the navy behind the badge's Tippy cells
PREVIEW_PAD = 60


def preview_svg(font, names, tiles=()) -> str:
    """Glyphs side by side at their advances; `tiles` get the navy cell color."""
    glyphs, hmtx = font.getGlyphSet(), font["hmtx"]
    bottom, top = BADGE_BOTTOM, BADGE_TOP
    paths, x = [], 0
    for name in names:
        advance = hmtx[name][0]
        if name in tiles:
            paths.append(f'<rect x="{x}" y="{bottom}" width="{advance}" height="{top - bottom}" fill="{PREVIEW_TILE}"/>')
        pen = SVGPathPen(glyphs)
        glyphs[name].draw(pen)
        paths.append(f'<path transform="translate({x} 0)" fill="{PREVIEW_INK}" d="{pen.getCommands()}"/>')
        x += advance
    pad = PREVIEW_PAD
    view = f"{-pad} {-top - pad} {x + 2 * pad} {top - bottom + 2 * pad}"
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{view}" height="64">'
        f'<g transform="scale(1 -1)">{"".join(paths)}</g></svg>\n'
    )


def write_previews(font) -> None:
    previews = HERE / "previews"
    previews.mkdir(exist_ok=True)
    for name, _, _, _ in GLYPHS:
        (previews / f"{name}.svg").write_text(preview_svg(font, [name]))
    badge = ["pill-left", "tippy-inverse-left", "tippy-inverse-right", "pill-right"]
    (previews / "prompt-badge.svg").write_text(
        preview_svg(font, badge, tiles={"tippy-inverse-left", "tippy-inverse-right"})
    )
    print(f"wrote {previews}/ ({len(GLYPHS) + 1} previews)")


if __name__ == "__main__":
    main()
