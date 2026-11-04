from palette import CREAM, INK
from pixels import rect

GLYPHS = {
    "a": [".....", ".....", ".###.", "....#", ".####", "#...#", ".####"],
    "d": ["....#", "....#", ".####", "#...#", "#...#", "#...#", ".####"],
    "i": ["..#..", ".....", ".##..", "..#..", "..#..", "..#..", ".###."],
    "m": [".....", ".....", "##.#.", "#.#.#", "#.#.#", "#.#.#", "#.#.#"],
    "v": [".....", ".....", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
    "w": [".....", ".....", "#...#", "#...#", "#.#.#", "#.#.#", ".#.#."],
    "y": [".....", ".....", "#...#", "#...#", ".####", "....#", ".###."],
    "b":["#....", "#....", "####.", "#...#", "#...#", "#...#", "####."],
    "c": [".....", ".....", ".###.", "#...#", "#....", "#...#", ".###."],
    "e": [".....", ".....", ".###.", "#...#", "#####", "#....", ".###."],
    "h": ["#....", "#....", "####.", "#...#", "#...#", "#...#", "#...#"],
    "l": [".##..", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "n": [".....", ".....", "####.", "#...#", "#...#", "#...#", "#...#"],
    "o": [".....", ".....", ".###.", "#...#", "#...#", "#...#", ".###."],
    "p": [".....", ".....", "####.", "#...#", "####.", "#....", "#...."],
    "r": [".....", ".....", "#.##.", "##..#", "#....", "#....", "#...."],
    "s": [".....", ".....", ".####", "#....", ".###.", "....#", "####."],
    "t": [".#...", ".#...", "####.", ".#...", ".#...", ".#..#", "..##."],
    "u": [".....", ".....", "#...#", "#...#", "#...#", "#..##", ".##.#"],
    "z": [".....", ".....", "#####", "...#.", "..#..", ".#...", "#####"],
    " ": [".....", ".....", ".....", ".....", ".....", ".....", "....."],
}


def paint_glyph(img, glyph, x, y, scale, color):
    for row, line in enumerate(glyph):
        for col, cell in enumerate(line):
            if cell == "#":
                px, py = x + col * scale, y + row * scale
                rect(img, px, py, px + scale, py + scale, color)


def outline(width):
    reach = range(-width, width + 1)
    return [(dx, dy) for dx in reach for dy in range(-width, width + 2) if (dx, dy) != (0, 0)]


def paint_text(img, text, x, y, scale=1, color=CREAM, shadow=INK):
    offsets = outline(max(1, scale // 2))
    for i, char in enumerate(text):
        left = x + i * 6 * scale
        for dx, dy in offsets:
            paint_glyph(img, GLYPHS[char], left + dx, y + dy, scale, shadow)
        paint_glyph(img, GLYPHS[char], left, y, scale, color)


def paint_title(img, title, subtitle, x, y, scale=3, color=CREAM, shadow=INK):
    paint_text(img, title, x, y, scale, color, shadow)
    paint_text(img, subtitle, x, y + scale * 9, scale // 2, color, shadow)
