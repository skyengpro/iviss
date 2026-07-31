import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ImageProcessor } from '@/utils/imageProcessor';
import * as viewfinderModule from '@/utils/viewfinder';
import { computeViewfinderCrop, VF_ASPECT } from '@/utils/viewfinder';

// Mock translation function
const mockT = vi.fn((key: string) => key) as any;
const originalImage = globalThis.Image;

// Native capture resolution — the fixed "sensor" size used by every test below.
const NATIVE_IMG_WIDTH = 1920;
const NATIVE_IMG_HEIGHT = 1080;

// Helper factory for canvas mocking
function setupCanvasMock(contextMethods = {}, returnsNullContext = false) {
  const mockContext = returnsNullContext
    ? null
    : {
        drawImage: vi.fn(),
        getImageData: vi.fn(() => ({ data: new Uint8ClampedArray(4) })),
        ...contextMethods,
      };

  const mockCanvas = {
    getContext: vi.fn(() => mockContext),
    toDataURL: vi.fn(() => 'data:image/jpeg;base64,mockdata'),
    width: 0,
    height: 0,
  };

  const originalCreateElement = document.createElement.bind(document);
  vi.spyOn(document, 'createElement').mockImplementation(
    (tagName: string, options?: ElementCreationOptions) => {
      if (tagName === 'canvas') return mockCanvas as any;
      return originalCreateElement(tagName, options);
    }
  );

  return { mockCanvas, mockContext };
}

// Helper factory for Image mocking
function setupImageMock(triggerLoad = true) {
  globalThis.Image = class extends originalImage {
    constructor() {
      super();
      // Auto-trigger load or error in next tick
      setTimeout(() => {
        this.width = NATIVE_IMG_WIDTH;
        this.height = NATIVE_IMG_HEIGHT;
        if (triggerLoad && this.onload) this.onload(new Event('load'));
        else if (!triggerLoad && this.onerror) this.onerror('error');
      }, 0);
    }
  } as any;

  return () => {
    globalThis.Image = originalImage;
  };
}

/** Mirrors ImageProcessor's private drawViewfinderCrop math exactly, so the
 * expected canvas size is derived from the same computeViewfinderCrop the
 * implementation calls — not a second, independently hand-computed formula. */
function expectedCropOutput(maxWidth: number, boxW: number, boxH: number) {
  const crop = computeViewfinderCrop(NATIVE_IMG_WIDTH, NATIVE_IMG_HEIGHT, boxW, boxH)!;
  const outWidth = Math.min(crop.sw, maxWidth);
  const outHeight = outWidth * (crop.sh / crop.sw);
  return { crop, width: Math.round(outWidth), height: Math.round(outHeight) };
}

function setWindowSize(width: number, height: number) {
  Object.defineProperty(globalThis, 'innerWidth', { value: width, writable: true });
  Object.defineProperty(globalThis, 'innerHeight', { value: height, writable: true });
}

describe('ImageProcessor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setWindowSize(1024, 768);
  });

  afterEach(() => {
    globalThis.Image = originalImage;
    vi.restoreAllMocks();
  });

  describe('validateCameroonPlate', () => {
    it('should format a valid plate as "XX ### XX"', () => {
      expect(ImageProcessor.validateCameroonPlate('CE128BC')).toBe('CE 128 BC');
      expect(ImageProcessor.validateCameroonPlate('CE 128 BC')).toBe('CE 128 BC');
      expect(ImageProcessor.validateCameroonPlate('CE-128-BC')).toBe('CE 128 BC');
    });

    it('should handle lowercase input', () => {
      expect(ImageProcessor.validateCameroonPlate('ce128bc')).toBe('CE 128 BC');
    });

    it('should return null for invalid plates', () => {
      expect(ImageProcessor.validateCameroonPlate('')).toBeNull();
      expect(ImageProcessor.validateCameroonPlate('CE12BC')).toBeNull();
      expect(ImageProcessor.validateCameroonPlate('CE12BCA')).toBeNull();
      expect(ImageProcessor.validateCameroonPlate('ABCDEFG')).toBeNull();
    });

    it('should keep the legacy return type while supporting multiple Cameroon formats', () => {
      expect(ImageProcessor.validateCameroonPlate('LT4568A')).toBe('LT 4568 A');
      expect(ImageProcessor.validateCameroonPlate('LTSR9652A')).toBe('LT SR 9652 A');
      expect(ImageProcessor.validateCameroonPlate('PA02RC521')).toBe('PA 02 RC 521');
      expect(ImageProcessor.validateCameroonPlate('SN1234')).toBe('SN 1234');
      expect(ImageProcessor.validateCameroonPlate('1234567')).toBe('1234567');
    });

    it('should classify valid Cameroon plate categories', () => {
      expect(ImageProcessor.classifyCameroonPlate('SO128BC')).toEqual({
        plate: 'SO128BC',
        formatted: 'SO 128 BC',
        category: 'civil_cemac',
      });
      expect(ImageProcessor.classifyCameroonPlate('IS245642RC')).toEqual({
        plate: 'IS245642RC',
        formatted: 'IS 245642 RC',
        category: 'special_investment',
      });
    });
  });

  describe('preprocessForPhoto (primary photo path)', () => {
    it('crops to the shared viewfinder geometry and encodes at 0.95', async () => {
      const spy = vi.spyOn(viewfinderModule, 'computeViewfinderCrop');
      const { mockCanvas, mockContext } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const expected = expectedCropOutput(1400, 1024, 768);
      const result = await ImageProcessor.preprocessForPhoto('data:image', mockT);

      expect(result).toBe('data:image/jpeg;base64,mockdata');
      expect(spy).toHaveBeenCalledWith(NATIVE_IMG_WIDTH, NATIVE_IMG_HEIGHT, 1024, 768);
      expect(mockCanvas.width).toBe(expected.width);
      expect(mockCanvas.height).toBe(expected.height);
      expect(mockContext.drawImage).toHaveBeenCalledWith(
        expect.any(HTMLImageElement),
        expected.crop.sx,
        expected.crop.sy,
        expected.crop.sw,
        expected.crop.sh,
        0,
        0,
        expected.width,
        expected.height
      );
      expect(mockCanvas.toDataURL).toHaveBeenCalledWith('image/jpeg', 0.95);

      restoreImg();
    });

    it('downscales when the native crop exceeds the 1400px ceiling', async () => {
      setWindowSize(4500, 2000); // wide box -> native crop well above 1400px
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const expected = expectedCropOutput(1400, 4500, 2000);
      expect(expected.crop.sw).toBeGreaterThan(1400); // sanity check on the fixture itself

      await ImageProcessor.preprocessForPhoto('data:image', mockT);

      expect(mockCanvas.width).toBe(1400);
      expect(mockCanvas.height).toBe(expected.height);

      restoreImg();
    });

    it('never upscales a native crop narrower than the 1400px ceiling', async () => {
      setWindowSize(500, 400); // narrow box -> native crop well below 1400px
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const expected = expectedCropOutput(1400, 500, 400);
      expect(expected.crop.sw).toBeLessThan(1400); // sanity check on the fixture itself

      await ImageProcessor.preprocessForPhoto('data:image', mockT);

      expect(mockCanvas.width).toBe(expected.width);
      expect(mockCanvas.width).toBeLessThan(1400);

      restoreImg();
    });

    it('rejects with errors.imageLoad when image errors', async () => {
      setupCanvasMock();
      const restoreImg = setupImageMock(false);

      await expect(ImageProcessor.preprocessForPhoto('data:image', mockT)).rejects.toThrow(
        'errors.imageLoad'
      );

      restoreImg();
    });

    it('rejects with errors.canvasContext when getContext returns null', async () => {
      setupCanvasMock({}, true);
      const restoreImg = setupImageMock(true);

      await expect(ImageProcessor.preprocessForPhoto('data:image', mockT)).rejects.toThrow(
        'errors.canvasContext'
      );

      restoreImg();
    });
  });

  describe('preprocessForPhotoCapture (fallback photo path)', () => {
    it('uses the shared viewfinder geometry — not its own hardcoded aspect', async () => {
      const spy = vi.spyOn(viewfinderModule, 'computeViewfinderCrop');
      const { mockCanvas, mockContext } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const expected = expectedCropOutput(1800, 1024, 768);
      const result = await ImageProcessor.preprocessForPhotoCapture('data:image', mockT);

      expect(result).toBe('data:image/jpeg;base64,mockdata');
      expect(spy).toHaveBeenCalledWith(NATIVE_IMG_WIDTH, NATIVE_IMG_HEIGHT, 1024, 768);
      expect(mockCanvas.width).toBe(expected.width);
      expect(mockCanvas.height).toBe(expected.height);
      // The previous implementation hardcoded vh = vw / 2.0 — assert the
      // actual output aspect matches VF_ASPECT (within rounding), not 2.0.
      expect(mockCanvas.width / mockCanvas.height).toBeCloseTo(VF_ASPECT, 1);
      expect(mockContext.drawImage).toHaveBeenCalledWith(
        expect.any(HTMLImageElement),
        expected.crop.sx,
        expected.crop.sy,
        expected.crop.sw,
        expected.crop.sh,
        0,
        0,
        expected.width,
        expected.height
      );
      expect(mockCanvas.toDataURL).toHaveBeenCalledWith('image/jpeg', 0.95);

      restoreImg();
    });

    it('downscales at a larger ceiling (1800px) than the primary path', async () => {
      setWindowSize(4500, 2000);
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const primary = expectedCropOutput(1400, 4500, 2000);
      const fallback = expectedCropOutput(1800, 4500, 2000);
      // Same crop, different ceiling -> fallback path preserves more detail.
      expect(fallback.width).toBeGreaterThan(primary.width);

      await ImageProcessor.preprocessForPhotoCapture('data:image', mockT);

      expect(mockCanvas.width).toBe(fallback.width);
      expect(mockCanvas.height).toBe(fallback.height);

      restoreImg();
    });

    it('rejects with errors.imageLoad when image errors', async () => {
      const restoreImg = setupImageMock(false);

      await expect(ImageProcessor.preprocessForPhotoCapture('data:image', mockT)).rejects.toThrow(
        'errors.imageLoad'
      );

      restoreImg();
    });

    it('rejects with errors.canvasContext when getContext returns null', async () => {
      setupCanvasMock({}, true);
      const restoreImg = setupImageMock(true);

      await expect(ImageProcessor.preprocessForPhotoCapture('data:image', mockT)).rejects.toThrow(
        'errors.canvasContext'
      );

      restoreImg();
    });
  });

  describe('cropToViewfinderFast (live path)', () => {
    it('uses the shared viewfinder geometry and encodes at 0.95 (LIVE_CROP_OPTIONS)', async () => {
      const spy = vi.spyOn(viewfinderModule, 'computeViewfinderCrop');
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const expected = expectedCropOutput(800, 1024, 768);
      const result = await ImageProcessor.cropToViewfinderFast('data:image', mockT);

      expect(result).toBe('data:image/jpeg;base64,mockdata');
      expect(spy).toHaveBeenCalledWith(NATIVE_IMG_WIDTH, NATIVE_IMG_HEIGHT, 1024, 768);
      expect(mockCanvas.width).toBe(expected.width);
      expect(mockCanvas.height).toBe(expected.height);
      expect(mockCanvas.toDataURL).toHaveBeenCalledWith('image/jpeg', 0.95);

      restoreImg();
    });

    it('caps the live crop at 800px even when the native crop is much wider', async () => {
      setWindowSize(4500, 2000);
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const expected = expectedCropOutput(800, 4500, 2000);
      expect(expected.crop.sw).toBeGreaterThan(800);

      await ImageProcessor.cropToViewfinderFast('data:image', mockT);

      expect(mockCanvas.width).toBe(800);
      expect(mockCanvas.height).toBe(expected.height);

      restoreImg();
    });
  });

  describe('assessImageQuality', () => {
    const checkW = 400;
    const checkH = Math.round(400 / VF_ASPECT);

    function pixelsWith(rgb: [number, number, number]) {
      const data = new Uint8ClampedArray(checkW * checkH * 4);
      for (let i = 0; i < data.length; i += 4) {
        data[i] = rgb[0];
        data[i + 1] = rgb[1];
        data[i + 2] = rgb[2];
        data[i + 3] = 255;
      }
      return data;
    }

    it('rejects dark captures before OCR processing', async () => {
      setupCanvasMock({
        getImageData: vi.fn(() => ({ data: pixelsWith([20, 20, 20]) })),
      });
      const restoreImg = setupImageMock(true);

      await expect(ImageProcessor.assessImageQuality('data:image', mockT)).resolves.toEqual({
        isAcceptable: false,
        feedback: 'mobileScan.qualityTooDark',
      });

      restoreImg();
    });

    it('rejects overexposed captures before OCR processing', async () => {
      setupCanvasMock({
        getImageData: vi.fn(() => ({ data: pixelsWith([240, 240, 240]) })),
      });
      const restoreImg = setupImageMock(true);

      await expect(ImageProcessor.assessImageQuality('data:image', mockT)).resolves.toEqual({
        isAcceptable: false,
        feedback: 'mobileScan.qualityTooBright',
      });

      restoreImg();
    });

    it('rejects low-variance blurry captures before OCR processing', async () => {
      setupCanvasMock({
        getImageData: vi.fn(() => ({ data: pixelsWith([120, 120, 120]) })),
      });
      const restoreImg = setupImageMock(true);

      await expect(ImageProcessor.assessImageQuality('data:image', mockT)).resolves.toEqual({
        isAcceptable: false,
        feedback: 'mobileScan.qualityTooBlurry',
      });

      restoreImg();
    });
  });
});
