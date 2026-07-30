#!/usr/bin/env python3
"""Where does the viewfinder box actually land when the agent frames the plate?

Locates the orange plate with the same HSV profile the backend uses
(photo_ocr_service::find_plate_bbox, "orange": h 10-30, s>=0.4, v>=0.4),
then emulates the agent centring that plate in the viewfinder box and
filling its width -- for the shipped aspect 3.5 and for candidate fixes.
"""
import sys
from PIL import Image

sys.path.insert(0, ".")
import binarize_replica as B


def orange_bbox(im):
    """Same profile as find_plate_bbox's first entry, global pixel extent."""
    rgb = im.convert("RGB")
    w, h = rgb.size
    px = rgb.load()
    min_x, min_y, max_x, max_y, n = w, h, -1, -1, 0
    for y in range(h):
        for x in range(w):
            r, g, b = (c / 255.0 for c in px[x, y])
            cmax, cmin = max(r, g, b), min(r, g, b)
            delta = cmax - cmin
            if delta > 0:
                if cmax == r:
                    hue = 60.0 * (((g - b) / delta) % 6.0)
                elif cmax == g:
                    hue = 60.0 * (((b - r) / delta) + 2.0)
                else:
                    hue = 60.0 * (((r - g) / delta) + 4.0)
            else:
                hue = 0.0
            if hue < 0:
                hue += 360.0
            s = 0.0 if cmax == 0 else delta / cmax
            if 10.0 <= hue <= 30.0 and s >= 0.4 and cmax >= 0.4:
                n += 1
                min_x, max_x = min(min_x, x), max(max_x, x)
                min_y, max_y = min(min_y, y), max(max_y, y)
    if max_x < 0:
        return None
    return min_x, min_y, max_x - min_x + 1, max_y - min_y + 1, n


def main():
    src = Image.open(sys.argv[1])
    W, H = src.size
    bx, by, bw, bh, n = orange_bbox(src)
    print(f"source {W}x{H}")
    print(f"orange plate bbox: x={bx} y={by} {bw}x{bh}  aspect={bw/bh:.2f}  "
          f"({n} px, {100*n/(W*H):.1f}% of frame)\n")

    print("Agent frames the plate so it fills the viewfinder width:")
    print(" aspect | box height | plate height | vertical overshoot")
    for aspect in (3.5, 4.0, 4.5, 4.7):
        box_h = bw / aspect
        over = box_h - bh
        flag = "  <-- SHIPPED" if aspect == 3.5 else ""
        print(f"  {aspect:4.2f}  | {box_h:9.0f}px | {bh:10d}px | "
              f"{over:+6.0f}px ({100*over/bh:+5.1f}%){flag}")

    # Render the shipped 3.5 box and a 4.5 candidate, both centred on the plate.
    cx, cy = bx + bw / 2, by + bh / 2
    for aspect, tag in ((3.5, "shipped_3.5"), (4.5, "fixed_4.5")):
        cw = bw
        ch = cw / aspect
        x0, y0 = int(cx - cw / 2), int(cy - ch / 2)
        box = src.crop((max(0, x0), max(0, y0),
                        min(W, x0 + int(cw)), min(H, y0 + int(ch))))
        out_w = min(box.size[0], B.LIVE_MAX_WIDTH)
        out_h = max(1, round(out_w / aspect))
        box = box.resize((out_w, out_h), Image.LANCZOS)
        path = f"framed_{tag}.jpg"
        box.convert("RGB").save(path, "JPEG", quality=B.LIVE_QUALITY)
        print(f"\nwrote {path}  ({out_w}x{out_h})")

        gray = B.to_luma8(Image.open(path))
        desk, deg = B.deskew(gray)
        print(f"  deskew angle = {deg:+.1f} deg")
        for c, morph in ((5, True), (15, True), (15, False)):
            B.run(desk, c, morph, ".", f"framed_{tag}_C{c}{'' if morph else '_nomorph'}")


if __name__ == "__main__":
    main()
