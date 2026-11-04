from pathlib import Path

from PIL import Image

import dividers
from heading import render_heading
from pixelate import upscale

ASSETS = Path(__file__).resolve().parent.parent / "assets"

HEADINGS = ["what it is", "how it works", "reaching your host", "building from source"]
ICONS = {"library": 7, "shelf": 1, "drive": 13, "offline": 10, "invite": 4}
ICON_SIZE = 32


def slug(text):
    return text.replace(" ", "-")


def icon(sprite):
    canvas = Image.new("RGBA", (ICON_SIZE, ICON_SIZE), (0, 0, 0, 0))
    canvas.alpha_composite(sprite, ((ICON_SIZE - sprite.width) // 2, ICON_SIZE - sprite.height - 2))
    return upscale(canvas, ICON_SIZE * 2)


def main():
    for text in HEADINGS:
        render_heading(text).save(ASSETS / f"h-{slug(text)}.png")
    sprites = dividers.load_sprites()
    for name, index in ICONS.items():
        icon(sprites[index]).save(ASSETS / f"icon-{name}.png")
    for seed in range(1, 5):
        dividers.compose(sprites, seed).save(ASSETS / f"divider-{seed}.png")
    print("ok")


if __name__ == "__main__":
    main()
