import sys
from pathlib import Path

from PIL import Image

from font import paint_title
from palette import CREAM, ORANGE, ORANGE_DARK, SKIN, WOOD_INK
from pixelate import pixelate, upscale

ART = Path(__file__).resolve().parent
SOURCE = ART / "source.png"
ASSETS = ART.parent / "assets"

GRID_WIDTH = 768
COLORS = 56
OUT_WIDTH = 1536
TITLE_AT = (64, 72)
TITLE_SCALE = 6
PLANK = (398, 132, 556, 141)


def render(grid_width=GRID_WIDTH, colors=COLORS):
    img = pixelate(Image.open(SOURCE), grid_width, colors, (ORANGE, ORANGE_DARK, SKIN))
    plain = img.copy()
    paint_title(img, "larder", "your own drive", *TITLE_AT, TITLE_SCALE, CREAM, WOOD_INK)
    return upscale(img, OUT_WIDTH), divider(plain)


def divider(img):
    plank = img.crop(PLANK)
    strip = Image.new("RGB", (img.width, plank.height))
    for x in range(0, img.width, plank.width):
        strip.paste(plank, (x, 0))
    return upscale(strip, OUT_WIDTH)


if __name__ == "__main__":
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else ASSETS
    out.mkdir(exist_ok=True)
    banner, line = render()
    banner.save(out / "banner.png")
    line.save(out / "divider.png")
    print(out)
