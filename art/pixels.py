def put(img, x, y, color):
    if len(color) == 3 and img.mode == "RGBA":
        color = color + (255,)
    if 0 <= x < img.width and 0 <= y < img.height:
        img.putpixel((x, y), color)


def rect(img, x0, y0, x1, y1, color):
    for y in range(y0, y1):
        for x in range(x0, x1):
            put(img, x, y, color)
