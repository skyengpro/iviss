#!/usr/bin/env python3
"""Faithful replica of the iviss OCR preprocessing chain, in pure PIL.

Reproduces, stage by stage:
  frontend  ImageProcessor.cropToViewfinder  (viewfinder crop + downscale + JPEG)
  backend   ocr_service::preprocess
              to_luma8
            -> contrast_stretch_percentile   (p2/p98)
            -> adaptive_threshold            (integral image, radius=h/8, C)
            -> is_light_on_dark ? invert     (central region)
            -> deskew                        (on binary, fill 255, rebinarise)
            -> morphology_open               (separable 3x3 erode then dilate)
            -> add_border(30, 255)

Writes every intermediate stage to <outdir> so the image Tesseract actually
receives can be inspected, plus a sweep over ADAPTIVE_C and a morphology
on/off comparison.

Usage:  python3 binarize_replica.py <plate-image> [outdir]
"""
import sys
import os
from PIL import Image

# ── constants mirrored from the Rust source ──────────────────────────────────
VIEWFINDER_ASPECT = 4.5          # utils/viewfinder.ts
LIVE_MAX_WIDTH, LIVE_QUALITY = 800, 95    # imageProcessor.ts LIVE_CROP_OPTIONS
PHOTO_MAX_WIDTH, PHOTO_QUALITY = 1600, 95  # imageProcessor.ts PHOTO_CROP_OPTIONS

ADAPTIVE_C = 5                   # ocr_service.rs
ADAPTIVE_RADIUS_DIVISOR = 8
ADAPTIVE_RADIUS_MIN, ADAPTIVE_RADIUS_MAX = 15, 100
OCR_BORDER_PX = 30
DESKEW_MAX_DEG, COARSE_STEP, FINE_STEP = 7.0, 2.0, 0.5
DESKEW_PROBE_WIDTH = 300
DESKEW_MIN_CORRECTION_RAD = 0.01

# `image` crate rgb -> luma uses BT.709 (SRGB_LUMA = [2126, 7152, 722]/10000)
LUMA_R, LUMA_G, LUMA_B = 0.2126, 0.7152, 0.0722


def to_luma8(im):
    rgb = im.convert("RGB")
    w, h = rgb.size
    out = Image.new("L", (w, h))
    src, dst = rgb.getdata(), []
    for r, g, b in src:
        dst.append(int(LUMA_R * r + LUMA_G * g + LUMA_B * b))
    out.putdata(dst)
    return out


def crop_to_viewfinder(im, max_width, quality, tmp_path):
    """Emulates cropToViewfinder for a plate that fills the viewfinder width."""
    w, h = im.size
    sw = w
    sh = min(h, round(sw / VIEWFINDER_ASPECT))
    sx, sy = (w - sw) // 2, (h - sh) // 2
    crop = im.crop((sx, sy, sx + sw, sy + sh))

    out_w = min(sw, max_width)
    out_h = max(1, round(out_w / VIEWFINDER_ASPECT))
    crop = crop.resize((out_w, out_h), Image.LANCZOS)
    crop.convert("RGB").save(tmp_path, "JPEG", quality=quality)   # the upload
    return Image.open(tmp_path)


def row_projection_variance(im):
    w, h = im.size
    px = list(im.getdata())
    sums = [sum(px[y * w:(y + 1) * w]) for y in range(h)]
    mean = sum(sums) / h
    return sum((s - mean) ** 2 for s in sums)


def _rot(im, deg):
    return im.rotate(deg, resample=Image.BILINEAR, expand=False, fillcolor=0)


def estimate_skew_deg(im):
    w, h = im.size
    probe = im
    if w > DESKEW_PROBE_WIDTH:
        probe = im.resize(
            (DESKEW_PROBE_WIDTH, max(1, round(h * DESKEW_PROBE_WIDTH / w))), Image.BILINEAR
        )

    def best_in(lo, hi, step):
        steps = round((hi - lo) / step)
        best, best_var = 0.0, float("-inf")
        for i in range(steps + 1):
            a = lo + step * i
            v = row_projection_variance(_rot(probe, a))
            if v > best_var:
                best_var, best = v, a
        return best

    coarse = best_in(-DESKEW_MAX_DEG, DESKEW_MAX_DEG, COARSE_STEP)
    return best_in(max(coarse - COARSE_STEP, -DESKEW_MAX_DEG),
                   min(coarse + COARSE_STEP, DESKEW_MAX_DEG), FINE_STEP)


def deskew(im):
    deg = estimate_skew_deg(im)
    if abs(deg) * 3.14159265 / 180.0 > DESKEW_MIN_CORRECTION_RAD:
        return _rot(im, deg), deg
    return im.copy(), 0.0


def contrast_stretch_percentile(im):
    px = list(im.getdata())
    if not px:
        return im.copy()
    hist = [0] * 256
    for p in px:
        hist[p] += 1
    drop = int(len(px) * 0.02)

    lo, c = 0, 0
    for i, f in enumerate(hist):
        c += f
        if c > drop:
            lo = i
            break
    hi, c = 255, 0
    for i in range(255, -1, -1):
        c += hist[i]
        if c > drop:
            hi = i
            break
    if hi <= lo:
        return im.copy()

    rng = float(hi - lo)
    lut = [max(0, min(255, int((i - lo) / rng * 255.0))) for i in range(256)]
    return im.point(lut)


def adaptive_radius_for(height):
    return max(ADAPTIVE_RADIUS_MIN, min(ADAPTIVE_RADIUS_MAX, height // ADAPTIVE_RADIUS_DIVISOR))


def adaptive_threshold(im, radius, c):
    w, h = im.size
    px = list(im.getdata())
    iw = w + 1
    integral = [0] * (iw * (h + 1))
    for y in range(h):
        row_sum = 0
        row_off, cur, prev = y * w, (y + 1) * iw, y * iw
        for x in range(w):
            row_sum += px[row_off + x]
            integral[cur + x + 1] = row_sum + integral[prev + x + 1]

    out = [0] * (w * h)
    for y in range(h):
        y1, y2 = max(0, y - radius), min(h, y + radius + 1)
        row_off = y * w
        for x in range(w):
            x1, x2 = max(0, x - radius), min(w, x + radius + 1)
            count = (x2 - x1) * (y2 - y1)
            s = (integral[y2 * iw + x2] - integral[y1 * iw + x2]
                 - integral[y2 * iw + x1] + integral[y1 * iw + x1])
            thr = max(0, (s // count) - c)
            out[row_off + x] = 255 if px[row_off + x] > thr else 0

    res = Image.new("L", (w, h))
    res.putdata(out)
    return res


def separable_3x3(im, op):
    w, h = im.size
    src = list(im.getdata())
    pick = min if op == "erode" else max

    horiz = list(src)
    for y in range(h):
        row = y * w
        for x in range(1, w - 1):
            horiz[row + x] = pick(src[row + x - 1], src[row + x], src[row + x + 1])

    out = list(src)
    for y in range(1, h - 1):
        row, prev, nxt = y * w, (y - 1) * w, (y + 1) * w
        for x in range(1, w - 1):
            out[row + x] = pick(horiz[prev + x], horiz[row + x], horiz[nxt + x])

    res = Image.new("L", (w, h))
    res.putdata(out)
    return res


def morphology_open(im):
    w, h = im.size
    if w < 3 or h < 3:
        return im.copy()
    return separable_3x3(separable_3x3(im, "erode"), "dilate")


def is_light_on_dark(im):
    """Central region only, matching ocr_service::is_light_on_dark."""
    w, h = im.size
    ix, iy = int(w * 0.2), int(h * 0.2)
    px = im.load()
    vals = [px[x, y] for y in range(iy, max(iy + 1, h - iy))
            for x in range(ix, max(ix + 1, w - ix))]
    return sum(1 for v in vals if v > 128) * 2 < len(vals)



def invert(im):
    return im.point(lambda p: 255 - p)


def add_border(im, b, color):
    w, h = im.size
    out = Image.new("L", (w + 2 * b, h + 2 * b), color)
    out.paste(im, (b, b))
    return out


def run(gray, c, morph, outdir, tag, verbose=True):
    """Mirrors ocr_service::preprocess: stretch -> threshold -> polarity ->
    deskew (binary, white fill) -> morphology -> border."""
    stretched = contrast_stretch_percentile(gray)
    radius = adaptive_radius_for(stretched.size[1])
    binary = adaptive_threshold(stretched, radius, c)

    lod = is_light_on_dark(binary)
    if lod:
        binary = invert(binary)

    deg = estimate_skew_deg(binary)
    if abs(deg) > 0.573:
        binary = binary.rotate(deg, resample=Image.BILINEAR, expand=False, fillcolor=255)
        binary = binary.point(lambda p: 255 if p >= 128 else 0)

    if morph:
        binary = morphology_open(binary)
    final = add_border(binary, OCR_BORDER_PX, 255)
    final.save(os.path.join(outdir, f"{tag}.png"))
    if verbose:
        dark = sum(1 for p in binary.getdata() if p < 128) / (binary.size[0] * binary.size[1])
        print(f"  {tag:32s} radius={radius:3d} C={c:2d} morph={str(morph):5s} "
              f"inverted={str(lod):5s} dark={dark*100:5.1f}%")
    return final


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    src_path = sys.argv[1]
    outdir = sys.argv[2] if len(sys.argv) > 2 else "binarize_out"
    os.makedirs(outdir, exist_ok=True)

    src = Image.open(src_path)
    print(f"source: {src.size[0]}x{src.size[1]}  {src_path}\n")

    for label, max_w, q in (("live", LIVE_MAX_WIDTH, LIVE_QUALITY),
                            ("photo", PHOTO_MAX_WIDTH, PHOTO_QUALITY)):
        upload_path = os.path.join(outdir, f"{label}_00_upload.jpg")
        crop = crop_to_viewfinder(src, max_w, q, upload_path)
        print(f"[{label}] upload {crop.size[0]}x{crop.size[1]} @ q{q} "
              f"({os.path.getsize(upload_path)/1024:.0f} KB)")

        gray = to_luma8(crop)
        gray.save(os.path.join(outdir, f"{label}_01_gray.png"))

        desk, deg = deskew(gray)
        desk.save(os.path.join(outdir, f"{label}_02_deskew.png"))
        print(f"[{label}] deskew angle = {deg:+.1f} deg")

        contrast_stretch_percentile(desk).save(os.path.join(outdir, f"{label}_03_stretch.png"))

        # what the pipeline actually produces today
        run(desk, ADAPTIVE_C, True, outdir, f"{label}_04_AS_SHIPPED_C{ADAPTIVE_C}")
        # sweep to settle the speckle question
        for c in (10, 15, 20):
            run(desk, c, True, outdir, f"{label}_05_C{c}")
        run(desk, ADAPTIVE_C, False, outdir, f"{label}_06_C{ADAPTIVE_C}_nomorph")
        print()

    print(f"done -> {outdir}/")
    print("inspect *_04_AS_SHIPPED_* : that is the exact image Tesseract receives today")


if __name__ == "__main__":
    main()
