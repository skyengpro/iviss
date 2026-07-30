"""Replicates ocr_service::estimate_skew_angle exactly, in PIL.

Goal: check whether the row-projection-variance search is biased by the
black (Luma([0])) fill that rotate_about_center inserts, on a plate-like
image (DARK glyphs on a LIGHT background) rather than on the test fixture
(bright bars on a black background, where the fill matches the background).
"""
from PIL import Image

DESKEW_MAX_DEG = 7.0
COARSE = 2.0
FINE = 0.5
PROBE_W = 300


def row_projection_variance(im):
    w, h = im.size
    px = list(im.getdata())
    sums = [sum(px[y * w:(y + 1) * w]) for y in range(h)]
    mean = sum(sums) / h
    return sum((s - mean) ** 2 for s in sums)


def rotated(im, deg):
    # imageproc rotate_about_center: same size, bilinear, out-of-bounds = Luma([0])
    return im.rotate(deg, resample=Image.BILINEAR, expand=False, fillcolor=0)


def best_angle_in(probe, lo, hi, step):
    steps = round((hi - lo) / step)
    best, best_var = 0.0, float("-inf")
    for i in range(steps + 1):
        a = lo + step * i
        v = row_projection_variance(rotated(probe, a))
        if v > best_var:
            best_var, best = v, a
    return best


def estimate_skew_deg(im):
    w, h = im.size
    if w > PROBE_W:
        im = im.resize((PROBE_W, max(1, round(h * PROBE_W / w))), Image.BILINEAR)
    coarse = best_angle_in(im, -DESKEW_MAX_DEG, DESKEW_MAX_DEG, COARSE)
    lo = max(coarse - COARSE, -DESKEW_MAX_DEG)
    hi = min(coarse + COARSE, DESKEW_MAX_DEG)
    return best_angle_in(im, lo, hi, FINE)


def plate(w, h, bg, fg):
    """Perfectly level plate: one band of glyph-like blocks across the middle."""
    im = Image.new("L", (w, h), bg)
    px = im.load()
    gh = int(h * 0.55)
    y0 = (h - gh) // 2
    gw = int(w * 0.075)
    gap = int(w * 0.03)
    x = int(w * 0.06)
    while x + gw < w * 0.94:
        for yy in range(y0, y0 + gh):
            for xx in range(x, x + gw):
                px[xx, yy] = fg
        x += gw + gap
    return im


def fixture(w, h):
    """The image used by test_estimate_skew_angle_*: bright bars on black."""
    im = Image.new("L", (w, h))
    px = im.load()
    for y in range(h):
        v = 255 if (y // 4) % 3 == 0 else 0
        for x in range(w):
            px[x, y] = v
    return im


print("input (all perfectly level, correct answer = 0.0 deg)")
print(f"  test fixture   (bright bars on black) -> {estimate_skew_deg(fixture(400, 160)):+.1f} deg")
print(f"  plate  dark-on-light 800x229          -> {estimate_skew_deg(plate(800, 229, 230, 30)):+.1f} deg")
print(f"  plate  dark-on-light 1600x457         -> {estimate_skew_deg(plate(1600, 457, 230, 30)):+.1f} deg")
print(f"  plate  light-on-dark 800x229          -> {estimate_skew_deg(plate(800, 229, 30, 230)):+.1f} deg")

im = plate(800, 229, 230, 30)
probe = im.resize((PROBE_W, round(229 * PROBE_W / 800)), Image.BILINEAR)
print("\nrow-projection variance vs angle, level dark-on-light plate:")
for a in (-7.0, -5.0, -3.0, -1.0, 0.0, 1.0, 3.0, 5.0, 7.0):
    print(f"   {a:+5.1f} deg -> {row_projection_variance(rotated(probe, a)):.3e}")
