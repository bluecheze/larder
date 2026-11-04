from PIL import Image

from font import paint_text
from palette import CREAM, WOOD_INK

SCALE = 3
PAD = 3
OUT_SCALE = 2


def render_heading(text, color=CREAM, outline=WOOD_INK):
    width = len(text) * 6 * SCALE + PAD * 2
    height = 7 * SCALE + PAD * 2 + 1
    img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    paint_text(img, text, PAD, PAD, SCALE, color, outline)
    return img.resize((width * OUT_SCALE, height * OUT_SCALE), Image.Resampling.NEAREST)
