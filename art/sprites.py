from collections import deque

from PIL import Image

WHITE_CUTOFF = 228
MIN_PIXELS = 400


def is_background(rgb):
    return min(rgb[:3]) >= WHITE_CUTOFF


def neighbors(index, width, height):
    x, y = index % width, index // width
    if x + 1 < width:
        yield index + 1
    if x > 0:
        yield index - 1
    if y + 1 < height:
        yield index + width
    if y > 0:
        yield index - width


def outside_mask(pixels, width, height):
    outside = bytearray(width * height)
    edge = [i for i in range(width * height)
            if (i % width in (0, width - 1) or i // width in (0, height - 1)) and is_background(pixels[i])]
    queue = deque(edge)
    for i in edge:
        outside[i] = 1
    while queue:
        index = queue.popleft()
        for n in neighbors(index, width, height):
            if not outside[n] and is_background(pixels[n]):
                outside[n] = 1
                queue.append(n)
    return outside


def components(pixels, width, height):
    taken = outside_mask(pixels, width, height)
    blobs = []
    for start in range(width * height):
        if taken[start]:
            continue
        taken[start] = 1
        queue, blob = deque([start]), []
        while queue:
            index = queue.popleft()
            blob.append(index)
            for n in neighbors(index, width, height):
                if not taken[n]:
                    taken[n] = 1
                    queue.append(n)
        if len(blob) >= MIN_PIXELS:
            blobs.append(blob)
    return blobs


def cut(pixels, width, blob):
    xs = [i % width for i in blob]
    ys = [i // width for i in blob]
    left, top = min(xs), min(ys)
    sprite = Image.new("RGBA", (max(xs) - left + 1, max(ys) - top + 1))
    for i in blob:
        sprite.putpixel((i % width - left, i // width - top), pixels[i] + (255,))
    return sprite


def extract(img):
    rgb = img.convert("RGB")
    pixels = list(rgb.getdata())
    blobs = components(pixels, rgb.width, rgb.height)
    return [cut(pixels, rgb.width, blob) for blob in blobs]
