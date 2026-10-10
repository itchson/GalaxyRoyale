"""Authoring source for original Royale campaign cells; Python stdlib only.

Ship silhouettes, badge emblems and comms masks are authored below. Running this
file bakes their occupied cells into the same indexed atlas schema as game art.
No raster textures are used by the engine. Nose-up ships share gameplay/UI cells.
"""
import json
import math
from pathlib import Path

OUT = Path(__file__).with_name("campaign-atlas.json")
NAMES = ["Dawnneedle", "Bulwark", "Nightrail", "Prism", "Windrunner", "Ironwake", "Voidcoil", "Sunspear", "Royale", "Haven Scout"]
THEMES = [
    ((245, 184, 86), (136, 78, 28), (255, 230, 169)),
    ((237, 99, 103), (124, 42, 59), (255, 189, 169)),
    ((167, 126, 241), (79, 52, 121), (221, 202, 255)),
    ((81, 229, 233), (28, 104, 132), (190, 246, 249)),
    ((104, 216, 162), (37, 76, 68), (184, 248, 190)),
    ((238, 149, 71), (121, 67, 33), (255, 218, 140)),
    ((229, 111, 205), (102, 43, 87), (255, 194, 239)),
    ((235, 209, 128), (117, 97, 50), (255, 246, 194)),
]
THEMES += [THEMES[7], THEMES[3]]

# Metal/ink is shared; each hull retains its champion's three-tone livery.
COMMON = {"k": (8, 13, 29), "h": (32, 44, 67), "H": (78, 103, 138),
          "L": (159, 190, 212), "w": (235, 244, 245), "g": (247, 199, 89),
          "p": (12, 25, 43), "j": (25, 43, 64), "J": (78, 103, 138), "m": (96, 47, 61)}

# Authored silhouettes: needle, shoulder cruiser, forked serpent, crystal fork,
# swept-wing runner, ram prow, void ring, spear, crown and compact scout.
HULLS = [
    [(16, 1), (18, 9), (19, 17), (28, 25), (27, 29), (18, 26), (17, 31), (14, 31), (13, 26), (4, 29), (3, 25), (12, 17), (13, 9)],
    [(12, 5), (19, 5), (21, 9), (27, 9), (29, 14), (29, 25), (25, 29), (20, 27), (19, 31), (12, 31), (11, 27), (6, 29), (2, 25), (2, 14), (4, 9), (10, 9)],
    [(7, 1), (12, 6), (16, 9), (19, 6), (24, 1), (26, 7), (23, 13), (20, 18), (12, 18), (8, 13), (5, 7)],
    [(16, 1), (20, 12), (28, 7), (31, 12), (24, 21), (19, 24), (16, 31), (12, 24), (7, 21), (0, 12), (3, 7), (11, 12)],
    [(15, 3), (18, 7), (20, 18), (29, 11), (31, 15), (24, 27), (19, 29), (17, 25), (15, 29), (11, 29), (9, 25), (1, 15), (3, 11), (11, 18)],
    [(8, 3), (23, 3), (27, 8), (25, 12), (29, 17), (29, 25), (24, 29), (19, 27), (18, 31), (13, 31), (12, 27), (7, 29), (2, 25), (2, 17), (6, 12), (4, 8)],
    [],
    [(15, 0), (17, 8), (19, 13), (25, 17), (26, 24), (23, 29), (20, 25), (18, 30), (13, 30), (11, 25), (8, 29), (5, 24), (6, 17), (12, 13), (14, 8)],
    [(3, 1), (10, 9), (15, 2), (22, 9), (28, 1), (27, 15), (23, 23), (19, 27), (18, 31), (13, 31), (12, 27), (8, 23), (4, 15)],
    [(15, 3), (18, 10), (20, 20), (25, 25), (22, 28), (16, 25), (9, 28), (6, 25), (11, 20), (13, 10)],
]

# Thirteen-cell heraldic emblems. Dots are transparent; each has a distinct
# occupied silhouette so its identity remains readable without color.
EMBLEMS = [
    ["......w......", "......a......", ".....aaa.....", ".....aaa.....", "....aaaaa....", "aa...aaa...aa", ".aaaaaaaaaaa.", "..aaaaaaaaa..", "...aaaaaaa...", "....aaaaa....", ".....aaa.....", ".....a.a.....", "....a...a...."],
    ["..wwwwwwwww..", ".aaaaaaaaaaa.", ".a...aaa...a.", ".a..aaaaa..a.", ".a.aaaaaaa.a.", ".a..aaaaa..a.", "..a..aaa..a..", "..a..aaa..a..", "...a.aaa.a...", "....aaaaa....", ".....aaa.....", "......a......", "......g......"],
    ["...aaaaaaa...", "..aawwww.aa..", ".aaa.....aaa.", ".aa.......aa.", "..aaa........", "...aaaaaaa...", ".....ww.aaa..", ".........aaa.", ".aa......aaa.", ".aaa....aaa..", "..aaaaaaaa...", "...aaaaaa....", "....g........"],
    ["......w......", ".....aaa.....", "....aa.aa....", "...aa...aa...", "..aa..a..aa..", ".aa..aaa..aa.", "aa..aaaaa..aa", ".aa..aaa..aa.", "..aa..a..aa..", "...aa...aa...", "....aa.aa....", ".....aaa.....", "......a......"],
    ["aa.........aa", ".aaa.....aaa.", "..aaa...aaa..", "...aaa.aaa...", "a...aaaaa...a", ".aa..aaa..aa.", "..aaaa.aaaa..", "...aa...aa...", "....a.w.a....", ".....aaa.....", "......a......", ".....a.a.....", "....a...a...."],
    ["...wwwwwww...", "..aaaaaaaaa..", ".aaaaaaaaaaa.", ".aa..aaa..aa.", ".....aaa.....", ".....aaa.....", ".....aaa.....", ".....aaa.....", "....aaaaa....", "...aaaaaaa...", "..aaaaaaaaa..", ".aaaaaaaaaaa.", "..g.......g.."],
    [".....aaa.....", "...aa...aa...", "..aa.....aa..", ".aa...w...aa.", "aa...aaa...aa", "a...aaaaa...a", "a...a.g.a...a", "a...aaaaa...a", "aa...aaa...aa", ".aa...w...aa.", "..aa.....aa..", "...aa...aa...", ".....aaa....."],
    ["g.....w.....g", ".a...aaa...a.", ".aa..aaa..aa.", "..aaaaaaaaa..", "..aaaaaaaaa..", "...aaaaaaa...", ".....aaa.....", ".....aaa.....", ".....aaa.....", ".....aaa.....", ".....aaa.....", "......a......", "......g......"],
]

# New dialogue silhouettes. Existing three dialogue portraits remain unchanged
# in Rust. All eye rows occupy y=11 for the same optional blink treatment.
PORTRAITS = {
    3: ["", "kak", "kaLaak", "kaLLLLak", "kaLLaaLLak", "kaLLFFFFLLak", "kaLFFFFFFFFLak", "kLFFFllFFllFFF Lk".replace(" ", ""), "kFFFllFFFFllFFFk", "kFFllFFFFFFllFFk", "kFFkkFFkkFFFFk", "kFFpeFFFFepFFk", "kFlFFFFFFFlFk", "kFFllFFllFFFk", "kFllFffFllFFk", "kFlFFffFFlFk", "kFllFFFFFFllFk", "kFFFFmmmFFFFk", "kFFFFFmFFFFFk", "kFFFFFFFFFFk", "kkFFFFFFkk", "kaLkffffkLak", "kaLLkwwkLLak", "kaLLLakkkLLLak", "kaLLLjjggjjLLLak", "kaLLLjjggjjLLLak", "kaLLjjjggjjjLLak", "kaLLjjjggjjjLLak", "kaLjjjwwwwjjjLak", "kaLjjjwwwwjjjLak", "kaLjjjjjjjjjjLak", "kkkkkkkkkkkkkkkkkkkkkkkk"],
    4: ["", "hhhhhhhh", "hhHHHHHHHHhh", "hHHHHHHHHHHHhh", "hHHHHHHHHHHHHHh", "hhHHHHHHHHHHHHHh", "hHHhhhkkkkHHHHHhh", "hHHhhFFFFFFFFkhHhh", "hHHhFlFFFFFFlFkhHh", "hhhkFllFFFFFFllFkhh", "hkFkkkFFkkkFFkhh", "hkFpeFFFFepFFkhh", "hkFlFFFFFFFlFkhh", "hkFlllFFlllFFkhh", "hkFllFffFllFFkhh", "hkFllFffFllFFkhh", "hkFlFFFFFFFlFkhh", "hkFFmmmFFFkhh", "hhkFFFmFFFFkhh", "hhh kFFFFFFk".replace(" ", ""), "hhkkFFFFkkhh", "hkwwkffffkwwkh", "kJJkwwwwkJJk", "kJJJJkwwkJJJJk", "kJJJJjjkkjjJJJJk", "kJJJJjjaggajjJJJJk", "kJJJJjjjggjjjJJJJk", "kJJJjjjjggjjjjJJJk", "kJJjjjjwwwwjjjjJJk", "kJJjjjjwwwwjjjjJJk", "kJJjjjjjjjjjjjjJJk", "kkkkkkkkkkkkkkkkkkkkkkkk"],
    5: ["", "kk kk".replace(" ", "a"), "kkHHkkkkHHkk", "kkHHHHHHHHHHkk", "kHHHHHHHHHHHHHHk", "kHHFFFHHFFFHHHHk", "kHFFFFFFFFFFFFHk", "kHFFffFFFFffFFHk", "kHFFFffFFffFFFHk", "kHFFFFFFFFFFFFHk", "kHkkkFFkkkFFFFHk", "kHFpeFFFFepFFHk", "kHFllFFFFllFFHk", "kHFFFFffFFFFFHk", "kHFffFffFffFFHk", "kHFFFffffFFFHk", "kHFFFFFFFFFFHk", "kHFFFmmmFFFFHk", "kHFFFFmFFFFFHk", "kHHFFFFFFFFHHk", "kkHHFFFFFFHHkk", "kHHHkffffkHHHk", "kJJJJkkkkJJJJk", "kJJJJJggJJJJJk", "kJJJJjjggjjJJJJk", "kJJJJjjggjjJJJJk", "kJJJjjjggjjjJJJk", "kJJJjjjggjjjJJJk", "kJJjjjwwwwjjjJJk", "kJJjjjwwwwjjjJJk", "kJJjjjjjjjjjjJJk", "kkkkkkkkkkkkkkkkkkkkkkkk"],
    6: ["", "kkkkkk", "kkhhhhhhhhkk", "khhhHHHHHHhhhk", "khhhHHHHHHHHhhhk", "khhhHHHHHHHHHhhhk", "khhhkkkFFkkkhhhk", "khhFFFFFFFFFFFFhhk", "khhFFFFFFFFFFFFhhk", "khhFFFFFFFFFFFFhhk", "khhFFkkFFkkFFFFhhk", "khhFFeeFFeeFFFFhhk", "khhFFFFFFFFFFFFhhk", "khhFFFFFFaFFFFFhhk", "khhFFFFaaaFFFFFhhk", "khhFFFFFaFFFFFFhhk", "khhFFFFFFFFFFFFhhk", "khhFFFmmmFFFFFFhhk", "khhFFFFmFFFFFFFhhk", "khhFFFFFFFFFFFFhhk", "khhkkFFFFFFkkhhk", "khhhkffffkhhhk", "khHHHkkkkHHHhk", "khHHHHaaHHHHhk", "khHHHjjggjjHHHhk", "khHHHjjggjjHHHhk", "khHHjjjggjjjHHhk", "khHHjjjggjjjHHhk", "khHjjjwwwwjjjHhk", "khHjjjwwwwjjjHhk", "khHjjjjjjjjjjHhk", "kkkkkkkkkkkkkkkkkkkkkkkk"],
    7: ["", "kkkkkkkk", "kkHHggggHHkk", "kHHHHggHHHHk", "kHHHHHHHHHHHHk", "kHHHHHHHHHHHHHHk", "kHhhkkkkkkhhHHk", "kHhFFFFFFFFFFhHk", "kHhFlFFFFFFlFhHk", "kHhFllFFFFFFllFhHk", "kHhFkkkFFkkkFFhHk", "kHhFpeFFFFepFFhHk", "kHhFlFFFFFFFlFhHk", "kHhFlllFFlllFFhHk", "kHhFllFffFllFFhHk", "kHhFllFffFllFFhHk", "kHhFlFFFFFFFlFhHk", "kHhFFmmmFFFhHk", "kHhFFFmFFFFhHk", "kHHFFFFFFHHk", "kkHHFFFFHHkk", "kHHkffffkHHk", "kJJkggggkJJk", "kJJJJkggkJJJJk", "kJJJJjjkkjjJJJJk", "kJJJJjjjggjjjJJJJk", "kJJJJjjjjggjjjjJJJJk", "kJJJjjjjjggjjjjjJJJk", "kJJjjjjjwwwwjjjjjJJk", "kJJjjjjjwwwwjjjjjJJk", "kJJjjjjjjjjjjjjjjJJk", "kkkkkkkkkkkkkkkkkkkkkkkk"],
}

def inside(x, y, vertices):
    hit = False
    for a, b in zip(vertices, vertices[1:] + vertices[:1]):
        if (a[1] > y) != (b[1] > y) and x < (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) + a[0]:
            hit = not hit
    return hit

def ship(style):
    occupied = {(x, y) for y in range(32) for x in range(32) if inside(x + .5, y + .5, HULLS[style])}
    if style == 2:
        for cy, rx, ry in [(21, 4.5, 2), (26, 3.5, 2), (30, 2.5, 1.5)]:
            occupied |= {(x, y) for y in range(32) for x in range(32) if ((x + .5 - 16) / rx) ** 2 + ((y + .5 - cy) / ry) ** 2 <= 1}
    if style == 6:
        occupied = {(x, y) for y in range(32) for x in range(32) if 7.2 <= math.hypot(x + .5 - 16, y + .5 - 16) <= 12}
        for fin in [[(14, 5), (16, 0), (18, 5)], [(27, 14), (32, 16), (27, 18)], [(14, 27), (16, 32), (18, 27)], [(5, 14), (0, 16), (5, 18)]]:
            occupied |= {(x, y) for y in range(32) for x in range(32) if inside(x + .5, y + .5, fin)}
    grid = [["." for _ in range(32)] for _ in range(32)]
    adjacent = lambda x, y: [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
    edge = {p for p in occupied if any(q not in occupied for q in adjacent(*p))}
    for x, y in occupied:
        grid[y][x] = "k" if (x, y) in edge else "a" if any(q in edge for q in adjacent(x, y)) else "H" if x + y < 34 else "h"
        if (x, y - 1) not in occupied and y < 18:
            grid[y][x] = "L"
    def paint(x, y, value):
        if (x, y) in occupied: grid[y][x] = value

    # Hardware is painted onto the existing occupied hull, never added as new
    # collision cells. Panels have a dark seam, a lit leading edge and separate
    # armor facets instead of a flat fill or decorative random noise.
    def detail(x, y, value):
        if (x, y) in occupied and (x, y) not in edge:
            grid[y][x] = value

    def rail(points, value):
        for (x, y), (xx, yy) in zip(points, points[1:]):
            dx, dy = abs(xx - x), -abs(yy - y)
            sx, sy = 1 if x < xx else -1, 1 if y < yy else -1
            error = dx + dy
            while True:
                detail(x, y, value)
                if (x, y) == (xx, yy): break
                twice = error * 2
                if twice >= dy: error += dy; x += sx
                if twice <= dx: error += dx; y += sy

    def panel(points, fill, highlight="L"):
        for y in range(32):
            for x in range(32):
                if inside(x + .5, y + .5, points): detail(x, y, fill)
        rail(points + points[:1], "p")
        rail(points[:2], highlight)

    def vent(x, y, width, rows):
        for row in range(rows):
            rail([(x, y + row * 2), (x + width - 1, y + row * 2)], "p")
            detail(x, y + row * 2 + 1, "L")
            detail(x + width - 1, y + row * 2 + 1, "H")

    def bolt(x, y):
        detail(x, y, "w"); detail(x + 1, y, "h")

    if style == 0:
        panel([(14, 7), (17, 7), (18, 20), (13, 20)], "H")
        panel([(11, 20), (13, 22), (6, 26), (5, 25)], "H")
        panel([(18, 22), (20, 20), (26, 25), (25, 26)], "h")
        rail([(15, 5), (15, 10)], "l")
        rail([(9, 24), (12, 22)], "l")
        rail([(20, 23), (23, 25)], "a")
        vent(14, 21, 4, 2); bolt(12, 25); bolt(20, 25)
    elif style == 1:
        panel([(5, 12), (10, 12), (11, 24), (6, 26), (4, 23)], "H")
        panel([(21, 12), (26, 12), (27, 23), (24, 26), (20, 24)], "h")
        panel([(12, 7), (18, 7), (20, 11), (11, 11)], "L", "w")
        panel([(12, 20), (19, 20), (19, 25), (16, 28), (12, 25)], "H")
        rail([(7, 14), (9, 14), (9, 21)], "a")
        rail([(22, 14), (24, 14), (24, 21)], "s")
        vent(6, 17, 4, 3); vent(22, 17, 4, 3); vent(13, 22, 5, 2)
        for p in [(6, 12), (24, 12), (11, 26), (20, 26)]: bolt(*p)
    elif style == 2:
        panel([(7, 5), (10, 7), (13, 13), (10, 14), (7, 9)], "L", "l")
        panel([(23, 5), (24, 9), (21, 14), (18, 13), (21, 7)], "H", "a")
        rail([(11, 10), (14, 12), (14, 16)], "s")
        rail([(20, 10), (17, 12), (17, 16)], "a")
        for cy, radius in [(21, 3), (26, 2), (30, 1)]:
            rail([(16 - radius, cy), (16 + radius, cy)], "s")
            rail([(16 - radius, cy - 1), (16 + radius - 1, cy - 1)], "l")
            detail(16, cy, "w")
    elif style == 3:
        panel([(4, 10), (10, 14), (8, 18), (3, 13)], "l", "w")
        panel([(27, 10), (29, 13), (23, 19), (20, 15)], "s", "a")
        panel([(13, 10), (16, 5), (18, 11), (16, 22), (13, 19)], "L", "w")
        rail([(4, 12), (10, 17)], "a")
        rail([(26, 12), (21, 17)], "l")
        rail([(12, 20), (15, 26), (17, 23)], "a")
        bolt(8, 17); bolt(22, 17)
    elif style == 4:
        panel([(3, 14), (6, 17), (12, 20), (11, 25), (7, 20)], "H", "l")
        panel([(27, 14), (29, 15), (23, 24), (20, 25), (19, 20)], "h", "a")
        panel([(13, 8), (17, 9), (18, 21), (12, 21)], "L", "w")
        rail([(5, 16), (10, 21)], "l")
        rail([(26, 17), (22, 22)], "a")
        vent(12, 22, 3, 2); vent(18, 22, 3, 2)
        bolt(9, 22); bolt(22, 24)
    elif style == 5:
        panel([(8, 5), (22, 5), (24, 8), (21, 10), (10, 10), (6, 8)], "L", "w")
        panel([(7, 14), (11, 15), (11, 23), (7, 26), (4, 23)], "H", "a")
        panel([(21, 15), (25, 14), (27, 23), (23, 26), (20, 23)], "h", "a")
        panel([(12, 20), (19, 20), (19, 25), (17, 28), (13, 28)], "s", "g")
        rail([(9, 7), (21, 7)], "g")
        rail([(10, 9), (21, 9)], "s")
        vent(10, 6, 3, 1); vent(18, 6, 3, 1)
        vent(6, 17, 4, 3); vent(22, 17, 4, 3)
        for p in [(8, 10), (22, 10), (7, 25), (23, 25)]: bolt(*p)
    elif style == 6:
        # Four segmented reactor arcs with a beveled inner edge and ports.
        for x, y in occupied - edge:
            radius = math.hypot(x + .5 - 16, y + .5 - 16)
            angle = math.atan2(y + .5 - 16, x + .5 - 16)
            sector = int((angle + math.pi) * 8 / math.tau) % 8
            grid[y][x] = ["H", "h", "s", "h"][sector % 4]
            if radius < 8.4: grid[y][x] = "l" if x < 16 else "a"
            elif radius > 10.6: grid[y][x] = "L" if y < 16 else "H"
        for x, y in [(15, 5), (25, 15), (15, 25), (5, 15)]:
            panel([(x - 1, y - 1), (x + 2, y - 1), (x + 2, y + 2), (x - 1, y + 2)], "a", "l")
            detail(x, y, "w")
        for p in [(9, 9), (22, 9), (9, 22), (22, 22)]: bolt(*p)
    elif style == 7:
        panel([(7, 19), (11, 17), (13, 22), (10, 25), (7, 23)], "L", "l")
        panel([(20, 17), (23, 19), (24, 23), (21, 25), (18, 22)], "H", "a")
        panel([(13, 11), (17, 11), (19, 21), (17, 27), (13, 27)], "s", "g")
        rail([(14, 8), (14, 13)], "L")
        rail([(18, 13), (19, 19)], "a")
        vent(8, 21, 3, 2); vent(21, 21, 3, 2)
        bolt(10, 18); bolt(21, 18)
    elif style == 8:
        # Warm gold armor carries chromatic inlays rather than flat grey wings.
        panel([(6, 8), (10, 11), (13, 16), (10, 22), (6, 14)], "a", "l")
        panel([(22, 11), (26, 8), (25, 15), (21, 22), (18, 16)], "s", "g")
        panel([(14, 8), (16, 7), (20, 20), (16, 27), (12, 21)], "H", "w")
        rail([(7, 11), (10, 14), (9, 19)], "l")
        rail([(23, 11), (22, 14), (22, 19)], "a")
        for x, y, hue in [(8, 13, "t"), (22, 13, "r"), (11, 19, "v"), (20, 19, "t")]:
            detail(x, y - 1, "w"); detail(x - 1, y, hue)
            detail(x, y, "l"); detail(x + 1, y, hue); detail(x, y + 1, "s")
        vent(13, 23, 5, 2); bolt(10, 21); bolt(22, 21)
    else:
        panel([(13, 11), (17, 11), (19, 21), (13, 22)], "L", "w")
        panel([(9, 23), (12, 22), (13, 25), (8, 26)], "H", "l")
        panel([(18, 22), (22, 24), (21, 26), (18, 25)], "h", "a")
        vent(14, 20, 4, 2)

    if style != 6:
        for y in range(12, 19):
            for x in range(14, 18): paint(x, y, "e" if x > 14 else "w")
        for y in range(26, 30):
            for x in [13, 18]: paint(x, y, "l" if y < 28 else "a")
    if style == 7:
        for y in range(1, 12):
            paint(15, y, "w"); paint(16, y, "g")
    if style == 8:
        for y in range(4, 10): paint(15, y, "g")
    return grid

def patch(style):
    corners = [(5, 1), (18, 1), (22, 5), (22, 18), (18, 22), (5, 22), (1, 18), (1, 5)]
    filled = {(x, y) for y in range(24) for x in range(24) if inside(x + .5, y + .5, corners)}
    grid = [["." for _ in range(24)] for _ in range(24)]
    for x, y in filled:
        if any((x + dx, y + dy) not in filled for dx, dy in [(2, 0), (-2, 0), (0, 2), (0, -2)]):
            grid[y][x] = "L" if y < 10 else "H"
        if any((x + dx, y + dy) not in filled for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]):
            grid[y][x] = "k"
    # The emblem floats inside its frame. Only its own ink outline is occupied.
    for y, row in enumerate(EMBLEMS[style]):
        for x, value in enumerate(row):
            if value != ".":
                for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
                    xx, yy = x + 5 + dx, y + 5 + dy
                    if grid[yy][xx] == ".": grid[yy][xx] = "k"
    for y, row in enumerate(EMBLEMS[style]):
        for x, value in enumerate(row):
            if value != ".": grid[y + 5][x + 5] = value
    return grid

palette = [0]
def index(rgb):
    value = rgb[0] | rgb[1] << 8 | rgb[2] << 16 | 255 << 24
    if value not in palette: palette.append(value)
    return palette.index(value)

def colors(style):
    accent, shadow, light = THEMES[style]
    skin = {4: ((214, 154, 118), (119, 71, 59), (249, 210, 168)),
            5: ((167, 132, 96), (93, 70, 47), (219, 190, 139)),
            6: ((61, 58, 92), (32, 44, 67), (156, 136, 194)),
            7: ((78, 103, 138), (32, 44, 67), (159, 190, 212))}.get(style, (accent, shadow, light))
    return {**COMMON, "a": accent, "s": shadow, "l": light, "F": skin[0], "f": skin[1],
            "e": light, "c": accent, "r": THEMES[6][0], "t": THEMES[3][0], "v": THEMES[2][0]}

sprites = {}
def add(name, grid, style):
    height, width = len(grid), len(grid[0])
    assert all(len(row) == width for row in grid), name
    table = {key: index(rgb) for key, rgb in colors(style).items()}
    indices = [0 if cell == "." else table[cell] for row in grid for cell in row]
    assert sum(value != 0 for value in indices) > 0, name
    sprites[name] = {"width": width, "height": height, "indices": indices}

for style in range(10): add(f"campaign_ship_{style}", ship(style), style)
for style in range(8): add(f"campaign_patch_{style}", patch(style), style)
for style, rows in PORTRAITS.items():
    assert len(rows) == 32 and all(len(row) <= 24 for row in rows), style
    grid = [["."] * 24 for _ in range(32)]
    for y, row in enumerate(rows):
        start = (24 - len(row)) // 2
        grid[y][start:start + len(row)] = row
    add(f"campaign_portrait_{style}", grid, style)

assert len(palette) <= 64, f"palette overflow: {len(palette)}"
palette += [0] * (64 - len(palette))
OUT.write_text(json.dumps({"version": 1, "palette": palette, "sprites": sprites}, separators=(",", ":")) + "\n", encoding="utf-8")
print(f"Baked {len(sprites)} authored sprites, {len(set(palette)) - 1} colors -> {OUT.name}")
