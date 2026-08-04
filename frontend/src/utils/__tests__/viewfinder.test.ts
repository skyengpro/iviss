import { describe, it, expect } from 'vitest';
import { computeViewfinderCrop, VF_ASPECT, VF_WIDTH_RATIO } from '../viewfinder';

describe('computeViewfinderCrop', () => {
  it('exposes the recommended aspect and width ratio', () => {
    expect(VF_ASPECT).toBe(4.5);
    expect(VF_WIDTH_RATIO).toBe(0.92);
  });

  it('centers a VF_ASPECT crop on the image at scale 1 (box matches image size)', () => {
    // box == image size -> object-cover scale is exactly 1
    const crop = computeViewfinderCrop(4500, 2000, 4500, 2000);

    expect(crop).toEqual({ sx: 180, sy: 540, sw: 4140, sh: 920 });
    expect(crop!.sw / crop!.sh).toBeCloseTo(VF_ASPECT);
  });

  it('scales the crop down when the image is smaller than the box (object-cover)', () => {
    // Same box as above, image at half resolution -> object-cover scale is 2
    const crop = computeViewfinderCrop(2250, 1000, 4500, 2000);

    expect(crop).toEqual({ sx: 90, sy: 270, sw: 2070, sh: 460 });
    expect(crop!.sw / crop!.sh).toBeCloseTo(VF_ASPECT);
  });

  it('clamps to the image bounds when the box aspect overflows the image on that axis', () => {
    // A box much wider (relative to its height) than the image forces the
    // VF_ASPECT-derived crop height past the image's own height.
    const crop = computeViewfinderCrop(1000, 100, 2000, 100);

    expect(crop).toEqual({ sx: 40, sy: 0, sw: 920, sh: 100 });
  });

  it('returns null for degenerate image dimensions', () => {
    expect(computeViewfinderCrop(0, 1080, 1024, 768)).toBeNull();
    expect(computeViewfinderCrop(1920, 0, 1024, 768)).toBeNull();
    expect(computeViewfinderCrop(-1, 1080, 1024, 768)).toBeNull();
  });

  it('returns null for degenerate box dimensions', () => {
    expect(computeViewfinderCrop(1920, 1080, 0, 768)).toBeNull();
    expect(computeViewfinderCrop(1920, 1080, 1024, 0)).toBeNull();
    expect(computeViewfinderCrop(1920, 1080, 1024, -1)).toBeNull();
  });
});
