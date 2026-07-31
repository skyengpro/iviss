import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { captureFrame, withTimeout, getImageCaptureCtor } from '../captureFrame';

function setupCanvasMock() {
  const mockContext = { drawImage: vi.fn() };
  const mockCanvas = {
    getContext: vi.fn(() => mockContext),
    toDataURL: vi.fn(() => 'data:image/jpeg;base64,bitmap'),
    width: 0,
    height: 0,
  };

  const originalCreateElement = document.createElement.bind(document);
  vi.spyOn(document, 'createElement').mockImplementation((tagName: string) => {
    if (tagName === 'canvas') return mockCanvas as unknown as HTMLCanvasElement;
    return originalCreateElement(tagName);
  });

  return { mockCanvas, mockContext };
}

class MockImageCapture {
  grabFrame = vi.fn();
  takePhoto = vi.fn();
  constructor(public track: MediaStreamTrack) {}
}

function fakeStream(): MediaStream {
  const track = {} as MediaStreamTrack;
  return {
    getVideoTracks: () => [track],
  } as unknown as MediaStream;
}

describe('getImageCaptureCtor', () => {
  afterEach(() => {
    delete (window as { ImageCapture?: unknown }).ImageCapture;
  });

  it('returns undefined when window.ImageCapture is not set', () => {
    expect(getImageCaptureCtor()).toBeUndefined();
  });

  it('returns the constructor when window.ImageCapture is set', () => {
    window.ImageCapture = MockImageCapture as never;
    expect(getImageCaptureCtor()).toBe(MockImageCapture);
  });
});

describe('withTimeout', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('resolves with the value when the promise settles before the timeout', async () => {
    const promise = withTimeout(Promise.resolve('value'), 1000);
    await expect(promise).resolves.toBe('value');
  });

  it('rejects once the timeout elapses before the promise settles', async () => {
    const never = new Promise(() => {});
    const promise = withTimeout(never, 1000);

    const assertion = expect(promise).rejects.toThrow('capture timed out');
    await vi.advanceTimersByTimeAsync(1000);
    await assertion;
  });
});

describe('captureFrame', () => {
  afterEach(() => {
    delete (window as { ImageCapture?: unknown }).ImageCapture;
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  it('falls back directly when there is no stream', async () => {
    const fallback = vi.fn(() => 'data:image/jpeg;base64,fallback');
    const result = await captureFrame(null, fallback);

    expect(result).toBe('data:image/jpeg;base64,fallback');
    expect(fallback).toHaveBeenCalledTimes(1);
  });

  it('falls back directly when ImageCapture is unsupported', async () => {
    const fallback = vi.fn(() => 'data:image/jpeg;base64,fallback');
    const result = await captureFrame(fakeStream(), fallback);

    expect(result).toBe('data:image/jpeg;base64,fallback');
    expect(fallback).toHaveBeenCalledTimes(1);
  });

  it('uses grabFrame() first when available', async () => {
    window.ImageCapture = MockImageCapture as never;
    setupCanvasMock();

    let capturedTrack: MediaStreamTrack | undefined;
    const originalCtor = MockImageCapture;
    class SpyImageCapture extends originalCtor {
      constructor(track: MediaStreamTrack) {
        super(track);
        capturedTrack = track;
        this.grabFrame.mockResolvedValue({ width: 100, height: 50 });
      }
    }
    window.ImageCapture = SpyImageCapture as never;

    const fallback = vi.fn(() => 'data:image/jpeg;base64,fallback');
    const stream = fakeStream();
    const result = await captureFrame(stream, fallback);

    expect(result).toBe('data:image/jpeg;base64,bitmap');
    expect(fallback).not.toHaveBeenCalled();
    expect(capturedTrack).toBe(stream.getVideoTracks()[0]);
  });

  it('falls back to takePhoto() when grabFrame() fails', async () => {
    class SpyImageCapture extends MockImageCapture {
      constructor(track: MediaStreamTrack) {
        super(track);
        this.grabFrame.mockRejectedValue(new Error('grabFrame unsupported'));
        this.takePhoto.mockResolvedValue(new Blob(['x'], { type: 'image/jpeg' }));
      }
    }
    window.ImageCapture = SpyImageCapture as never;

    const fallback = vi.fn(() => 'data:image/jpeg;base64,fallback');
    const result = await captureFrame(fakeStream(), fallback);

    expect(result).toMatch(/^data:/);
    expect(fallback).not.toHaveBeenCalled();
  });

  it('falls back to getScreenshot() when both grabFrame() and takePhoto() fail', async () => {
    class SpyImageCapture extends MockImageCapture {
      constructor(track: MediaStreamTrack) {
        super(track);
        this.grabFrame.mockRejectedValue(new Error('grabFrame unsupported'));
        this.takePhoto.mockRejectedValue(new Error('takePhoto unsupported'));
      }
    }
    window.ImageCapture = SpyImageCapture as never;

    const fallback = vi.fn(() => 'data:image/jpeg;base64,fallback');
    const result = await captureFrame(fakeStream(), fallback);

    expect(result).toBe('data:image/jpeg;base64,fallback');
    expect(fallback).toHaveBeenCalledTimes(1);
  });

  it('falls back to getScreenshot() when takePhoto() hangs past the timeout', async () => {
    vi.useFakeTimers();

    class SpyImageCapture extends MockImageCapture {
      constructor(track: MediaStreamTrack) {
        super(track);
        this.grabFrame.mockRejectedValue(new Error('grabFrame unsupported'));
        this.takePhoto.mockReturnValue(new Promise(() => {}));
      }
    }
    window.ImageCapture = SpyImageCapture as never;

    const fallback = vi.fn(() => 'data:image/jpeg;base64,fallback');
    const resultPromise = captureFrame(fakeStream(), fallback);

    await vi.advanceTimersByTimeAsync(1500);
    const result = await resultPromise;

    expect(result).toBe('data:image/jpeg;base64,fallback');
    expect(fallback).toHaveBeenCalledTimes(1);
  });
});
