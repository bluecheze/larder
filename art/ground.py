import random

from pixels import put, rect

PLANK_TOP = (196, 140, 84)
PLANK = (150, 98, 54)
PLANK_DARK = (96, 58, 30)
PLANK_EDGE = (52, 30, 16)
BRACKET = (72, 70, 74)


def paint_ground(img, top, seed):
    rng = random.Random(seed)
    width = img.width
    rect(img, 0, top, width, top + 1, PLANK_TOP)
    rect(img, 0, top + 1, width, top + 4, PLANK)
    rect(img, 0, top + 4, width, top + 5, PLANK_DARK)
    rect(img, 0, top + 5, width, top + 6, PLANK_EDGE)
    for _ in range(width // 9):
        x, y = rng.randrange(width), top + rng.randint(1, 3)
        for dx in range(rng.randint(3, 9)):
            put(img, x + dx, y, PLANK_DARK)
    for x in range(40, width, 190):
        paint_bracket(img, x + rng.randint(-12, 12), top + 6)


def paint_bracket(img, x, y):
    rect(img, x, y, x + 2, y + 5, BRACKET)
    for step in range(4):
        put(img, x + 2 + step, y + step, BRACKET)
