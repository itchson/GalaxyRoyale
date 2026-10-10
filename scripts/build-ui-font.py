"""Bake semantic-browser font metrics from the authoritative RoyaleMono cells.

Rendering stays in Rust. This small font makes hidden accessible controls measure
the same 5x7 glyphs and six-cell advances as the actual quad objects.
Requires fonttools only when authoring; the resulting TTF ships with the game.
"""
from pathlib import Path
import re
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "crates/game_web/src/pixel_ui.rs").read_text(encoding="utf-8")
masks = {}
for character, numbers in re.findall(r"'((?:\\.|[^']))'\s*=>\s*\[([0-9, ]+)\]", source):
    character = "'" if character == "\\'" else character
    rows = [int(value.strip()) for value in numbers.split(",")]
    if len(rows) == 7:
        masks[character] = rows
assert len(masks) >= 50
font = FontBuilder(1000, isTTF=True)
glyphs = {".notdef": TTGlyphPen(None).glyph()}
cmap = {}
for character in range(32, 127):
    name = f"cell{character}"
    pen = TTGlyphPen(None)
    for y, row in enumerate(masks.get(chr(character).upper(), [0] * 7)):
        for x in range(5):
            if row & (1 << (4 - x)):
                left, bottom = x * 125, (6 - y) * 125
                pen.moveTo((left, bottom)); pen.lineTo((left + 125, bottom))
                pen.lineTo((left + 125, bottom + 125)); pen.lineTo((left, bottom + 125))
                pen.closePath()
    glyphs[name] = pen.glyph()
    cmap[character] = name
font.setupGlyphOrder(list(glyphs))
font.setupCharacterMap(cmap)
font.setupGlyf(glyphs)
font.setupHorizontalMetrics({name: (750, 0) for name in glyphs})
font.setupHorizontalHeader(ascent=875, descent=-125)
font.setupNameTable({"familyName":"RoyaleMono", "styleName":"Regular", "uniqueFontIdentifier":"Galaxy Royale RoyaleMono 1.0", "fullName":"RoyaleMono Regular", "psName":"RoyaleMono-Regular", "version":"Version 1.0"})
font.setupOS2(sTypoAscender=875, sTypoDescender=-125, sTypoLineGap=0, usWinAscent=875, usWinDescent=125)
font.setupPost()
font.setupMaxp()
font.font["head"].created = font.font["head"].modified = 3853440000
output = ROOT / "apps/game/public/fonts/RoyaleMono.ttf"
font.save(output)
print(f"RoyaleMono: {len(cmap)} characters / {output.stat().st_size} bytes")
