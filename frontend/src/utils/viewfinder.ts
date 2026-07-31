export interface ViewfinderCrop {
  sx: number;
  sy: number;
  sw: number;
  sh: number;
}

// 92% of the display box width, 4.5:1 aspect — ~2% margin over the 4.7 CEMAC
// plate ratio (measured on samples/reference_plate_CE568LR.png).
export const VF_WIDTH_RATIO = 0.92;
export const VF_ASPECT = 4.5;

/**
 * Maps the on-screen viewfinder frame back onto source image pixels.
 * `boxW`/`boxH` is the display box the image covers (object-cover), e.g.
 * the video/container dimensions. Single source of truth for both the
 * overlay drawn on screen and the crop actually sent for OCR.
 * Returns null on degenerate dimensions — callers fall back to the full image.
 */
export function computeViewfinderCrop(
  imgW: number,
  imgH: number,
  boxW: number,
  boxH: number
): ViewfinderCrop | null {
  if (imgW <= 0 || imgH <= 0 || boxW <= 0 || boxH <= 0) return null;

  // object-cover: the image scales up to fully cover the box, cropping overflow
  const scale = Math.max(boxW / imgW, boxH / imgH);

  const vw = boxW * VF_WIDTH_RATIO;
  const vh = vw / VF_ASPECT;

  const sw = vw / scale;
  const sh = vh / scale;
  if (sw <= 0 || sh <= 0) return null;

  const sx = Math.max(0, (imgW - sw) / 2);
  const sy = Math.max(0, (imgH - sh) / 2);

  return {
    sx,
    sy,
    sw: Math.min(sw, imgW - sx),
    sh: Math.min(sh, imgH - sy),
  };
}
