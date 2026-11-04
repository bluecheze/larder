from collections import Counter

from PIL import Image

INK_SHARE = 0.25


def luminance(rgb):
    r, g, b = rgb
    return 0.299 * r + 0.587 * g + 0.114 * b


def palette_colors(indexed):
    flat = indexed.getpalette()
    return [tuple(flat[i:i + 3]) for i in range(0, len(flat), 3)]


def pick_cell(counts, total, brightness):
    darkest = min(counts, key=lambda index: brightness[index])
    if counts[darkest] >= total * INK_SHARE:
        return darkest
    return counts.most_common(1)[0][0]


def reduce_cells(indexed, grid_width, grid_height):
    cell_w = indexed.width / grid_width
    cell_h = indexed.height / grid_height
    brightness = [luminance(color) for color in palette_colors(indexed)]
    data = indexed.load()
    small = Image.new("P", (grid_width, grid_height))
    small.putpalette(indexed.getpalette())
    out = small.load()
    for gy in range(grid_height):
        for gx in range(grid_width):
            x0, y0 = int(gx * cell_w), int(gy * cell_h)
            x1, y1 = int((gx + 1) * cell_w), int((gy + 1) * cell_h)
            counts = Counter(data[x, y] for y in range(y0, y1) for x in range(x0, x1))
            out[gx, gy] = pick_cell(counts, (x1 - x0) * (y1 - y0), brightness)
    return small


def palette_with_accents(rgb, colors, accents):
    count = colors - len(accents)
    adaptive = rgb.quantize(colors=count, method=Image.Quantize.MEDIANCUT, kmeans=3)
    flat = adaptive.getpalette()[: 3 * count]
    for accent in accents:
        flat += list(accent)
    holder = Image.new("P", (1, 1))
    holder.putpalette(flat + flat[:3] * (256 - len(flat) // 3))
    return holder


def pixelate(img, grid_width, colors, accents=()):
    grid_height = round(grid_width * img.height / img.width)
    rgb = img.convert("RGB")
    palette = palette_with_accents(rgb, colors, accents)
    indexed = rgb.quantize(palette=palette, dither=Image.Dither.NONE)
    return reduce_cells(indexed, grid_width, grid_height).convert("RGB")


def upscale(img, width):
    scale = width // img.width
    return img.resize((img.width * scale, img.height * scale), Image.Resampling.NEAREST)
