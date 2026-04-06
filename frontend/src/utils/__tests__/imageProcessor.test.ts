import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ImageProcessor } from '@/utils/imageProcessor';

// Mock translation function
const mockT = vi.fn((key: string) => key) as any;

// Helper factory for canvas mocking
function setupCanvasMock(contextMethods = {}, returnsNullContext = false) {
  const mockContext = returnsNullContext
    ? null
    : {
        drawImage: vi.fn(),
        getImageData: vi.fn(() => ({ data: new Uint8ClampedArray(4) })),
        createImageData: vi.fn(() => ({ data: new Uint8ClampedArray(4) })),
        putImageData: vi.fn(),
        fillRect: vi.fn(),
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
  const originalImage = globalThis.Image;

  globalThis.Image = class extends originalImage {
    constructor() {
      super();
      // Auto-trigger load or error in next tick
      setTimeout(() => {
        this.width = 1920;
        this.height = 1080;
        if (triggerLoad && this.onload) this.onload(new Event('load'));
        else if (!triggerLoad && this.onerror) this.onerror('error');
      }, 0);
    }
  } as any;

  return () => {
    globalThis.Image = originalImage;
  };
}

describe('ImageProcessor', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Mock window inner width/height
    Object.defineProperty(globalThis, 'innerWidth', { value: 1024, writable: true });
    Object.defineProperty(globalThis, 'innerHeight', { value: 768, writable: true });
  });

  afterEach(() => {
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
      expect(ImageProcessor.validateCameroonPlate('1234567')).toBeNull();
    });
  });

  describe('preprocessForOCR', () => {
    it('resolves to a data URL when image loads', async () => {
      setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const result = await ImageProcessor.preprocessForOCR('data:image/jpeg;base64,...', mockT);
      expect(result).toBe('data:image/jpeg;base64,mockdata');

      restoreImg();
    });

    it('rejects with errors.imageLoad when image errors', async () => {
      const restoreImg = setupImageMock(false);

      await expect(ImageProcessor.preprocessForOCR('data:image', mockT)).rejects.toThrow(
        'errors.imageLoad'
      );

      restoreImg();
    });

    it('rejects with errors.canvasContext when getContext returns null', async () => {
      setupCanvasMock({}, true);
      const restoreImg = setupImageMock(true);

      await expect(ImageProcessor.preprocessForOCR('data:image', mockT)).rejects.toThrow(
        'errors.canvasContext'
      );

      restoreImg();
    });
  });

  describe('preprocessForHighRes', () => {
    it('resolves to a data URL', async () => {
      setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const result = await ImageProcessor.preprocessForHighRes('data:image', mockT);
      expect(result).toBe('data:image/jpeg;base64,mockdata');

      restoreImg();
    });

    it('rejects with errors.imageLoad when image errors', async () => {
      const restoreImg = setupImageMock(false);

      await expect(ImageProcessor.preprocessForHighRes('data:image', mockT)).rejects.toThrow(
        'errors.imageLoad'
      );

      restoreImg();
    });

    it('rejects with errors.canvasContext when getContext returns null', async () => {
      setupCanvasMock({}, true);
      const restoreImg = setupImageMock(true);

      await expect(ImageProcessor.preprocessForHighRes('data:image', mockT)).rejects.toThrow(
        'errors.canvasContext'
      );

      restoreImg();
    });
  });

  describe('preprocessForPhotoCapture', () => {
    it('resolves to a data URL and applies sharpening kernel', async () => {
      const { mockContext, mockCanvas } = setupCanvasMock({
        getImageData: vi.fn(() => ({ data: new Uint8ClampedArray(4 * 4 * 4) })), // Small mock image
        createImageData: vi.fn(() => ({ data: new Uint8ClampedArray(4 * 4 * 4) })),
      });
      const restoreImg = setupImageMock(true);

      const result = await ImageProcessor.preprocessForPhotoCapture('data:image', mockT);

      expect(result).toBe('data:image/jpeg;base64,mockdata');
      expect(mockContext.getImageData).toHaveBeenCalled();
      expect(mockContext.putImageData).toHaveBeenCalled();
      expect(mockCanvas.toDataURL).toHaveBeenCalledWith('image/jpeg', 0.92);

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

  describe('cropToViewfinder', () => {
    it('resolves and uses window.innerWidth/innerHeight correctly', async () => {
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const result = await ImageProcessor.cropToViewfinder('data:image', mockT);
      expect(result).toBe('data:image/jpeg;base64,mockdata');
      expect(mockCanvas.width).toBe(1200);
      expect(mockCanvas.height).toBe(600);

      restoreImg();
    });
  });

  describe('cropToViewfinderFast', () => {
    it('requests lower quality JPEG output (0.65)', async () => {
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      const result = await ImageProcessor.cropToViewfinderFast('data:image', mockT);

      expect(result).toBe('data:image/jpeg;base64,mockdata');
      expect(mockCanvas.width).toBe(800);
      expect(mockCanvas.height).toBe(400);
      expect(mockCanvas.toDataURL).toHaveBeenCalledWith('image/jpeg', 0.65);

      restoreImg();
    });
  });

  describe('scaleImage', () => {
    it('uses correct targetWidth scaling math', async () => {
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      // Default targetWidth is 800
      const result = await ImageProcessor.scaleImage('data:image', mockT);

      expect(result).toBe('data:image/jpeg;base64,mockdata');
      expect(mockCanvas.width).toBe(800);
      // Height should be scaled based on aspect ratio (1080 / (1920/800))
      expect(mockCanvas.height).toBe(1080 * (800 / 1920));

      restoreImg();
    });

    it('uses custom targetWidth', async () => {
      const { mockCanvas } = setupCanvasMock();
      const restoreImg = setupImageMock(true);

      await ImageProcessor.scaleImage('data:image', mockT, 400);

      expect(mockCanvas.width).toBe(400);
      expect(mockCanvas.height).toBe(1080 * (400 / 1920));

      restoreImg();
    });
  });
});
