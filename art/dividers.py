import random
import sys
from pathlib import Path

from PIL import Image

from ground import paint_ground
from pixelate import pixelate, upscale
from sprites import extract

ART = Path(__file__).resolve().parent
SHEET = ART / "sprites.png"
ASSETS = ART.parent / "assets"

WIDTH = 768
HEIGHT = 40
SHELF_Y = 32
OUT_WIDTH = 1536
TALLEST = 26
SHORTEST = 12


def shrink(sprite, target_height):
    flat = Image.new("RGB", sprite.size, (255, 255, 255))
    flat.paste(sprite, mask=sprite.split()[3])
    grid_width = max(4, round(sprite.width * target_height / sprite.height))
    small = pixelate(flat, grid_width, 20)
    alpha = sprite.split()[3].resize(small.size, Image.Resampling.BOX)
    out = small.convert("RGBA")
    out.putalpha(alpha.point(lambda a: 255 if a > 140 else 0))
    return out


def load_sprites():
    sprites = extract(Image.open(SHEET))
    biggest = max(s.height for s in sprites)
    sized = []
    for sprite in sprites:
        height = round(SHORTEST + (TALLEST - SHORTEST) * sprite.height / biggest)
        sized.append(shrink(sprite, height))
    return sized


def compose(sprites, seed):
    rng = random.Random(seed)
    strip = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
    x = rng.randint(6, 40)
    while x < WIDTH - 8:
        sprite = rng.choice(sprites)
        if x + sprite.width > WIDTH:
            break
        strip.alpha_composite(sprite, (x, SHELF_Y - sprite.height))
        x += sprite.width + rng.choice([1, 2, 3, 10, 24, 48])
    paint_ground(strip, SHELF_Y, seed)
    return upscale(strip, OUT_WIDTH)


if __name__ == "__main__":
    count = int(sys.argv[1]) if len(sys.argv) > 1 else 4
    sprites = load_sprites()
    for seed in range(1, count + 1):
        compose(sprites, seed).save(ASSETS / f"divider-{seed}.png")
    print(f"{len(sprites)} sprites, {count} dividers")
