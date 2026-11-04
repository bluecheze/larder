import random

from font import paint_title

CHALK = (232, 234, 222)
GRAIN = 0.14


def chalk_title(img, title, subtitle, x, y, scale):
    board = img.getpixel((x - 4, y - 4))
    paint_title(img, title, subtitle, x, y, scale, CHALK, None)
    roughen(img, (x, y, x + len(subtitle) * 6 * (scale // 2) + scale, y + scale * 16), board)


def roughen(img, box, board):
    rng = random.Random(7)
    left, top, right, bottom = box
    for py in range(top, min(bottom, img.height)):
        for px in range(left, min(right, img.width)):
            if img.getpixel((px, py)) == CHALK and rng.random() < GRAIN:
                img.putpixel((px, py), board)
