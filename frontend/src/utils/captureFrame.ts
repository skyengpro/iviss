// Minimal ambient typings for the ImageCapture API — not in TypeScript's lib.dom.d.ts.
interface ImageCapture {
  grabFrame(): Promise<ImageBitmap>;
  takePhoto(): Promise<Blob>;
}

interface ImageCaptureConstructor {
  new (videoTrack: MediaStreamTrack): ImageCapture;
}

declare global {
  interface Window {
    ImageCapture?: ImageCaptureConstructor;
  }
}

const TAKE_PHOTO_TIMEOUT_MS = 1500;

export function getImageCaptureCtor(): ImageCaptureConstructor | undefined {
  return typeof window !== 'undefined' ? window.ImageCapture : undefined;
}

export function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('capture timed out')), ms);
    promise.then(
      (value) => {
        clearTimeout(timer);
        resolve(value);
      },
      (error) => {
        clearTimeout(timer);
        reject(error);
      }
    );
  });
}

function bitmapToDataUrl(bitmap: ImageBitmap): string | null {
  const canvas = document.createElement('canvas');
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  const ctx = canvas.getContext('2d');
  if (!ctx) return null;
  ctx.drawImage(bitmap, 0, 0);
  return canvas.toDataURL('image/jpeg', 0.95);
}

function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(reader.error ?? new Error('failed to read captured photo'));
    reader.readAsDataURL(blob);
  });
}

/**
 * Capture scale for a real still, cheapest/fastest first:
 * 1. grabFrame() — next video frame at track resolution, no camera reconfig.
 * 2. takePhoto() — true still capture, but can reconfigure the sensor
 *    (re-focus/re-crop) so it's bounded by a timeout rather than left first.
 * 3. getScreenshot() fallback — Safari/iOS, no ImageCapture support.
 */
export async function captureFrame(
  stream: MediaStream | null,
  fallback: () => string | null
): Promise<string | null> {
  const track = stream?.getVideoTracks()[0];
  const ImageCaptureCtor = getImageCaptureCtor();

  if (track && ImageCaptureCtor) {
    const imageCapture = new ImageCaptureCtor(track);

    try {
      const bitmap = await imageCapture.grabFrame();
      const dataUrl = bitmapToDataUrl(bitmap);
      if (dataUrl) return dataUrl;
    } catch {
      // grabFrame unsupported or failed — fall through to takePhoto
    }

    try {
      const blob = await withTimeout(imageCapture.takePhoto(), TAKE_PHOTO_TIMEOUT_MS);
      return await blobToDataUrl(blob);
    } catch {
      // takePhoto unsupported, timed out, or failed — fall through to getScreenshot
    }
  }

  return fallback();
}
