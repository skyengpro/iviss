import { TFunction } from 'i18next';

export interface CameroonPlateClassification {
  plate: string;
  formatted: string;
  category: string;
}

const REGION = '(?:AD|CE|EN|ES|LT|NO|NW|OU|SU|SW|SO)';
const PLATE_PATTERNS: Array<{ category: string; regex: RegExp }> = [
  { category: 'trailer', regex: new RegExp(`^${REGION}(?:RE|SR|SE|TR)\\d{1,4}[A-Z]{1,2}$`) },
  { category: 'civil_cemac', regex: new RegExp(`^${REGION}\\d{3}[A-Z]{2}$`) },
  { category: 'civil_legacy', regex: new RegExp(`^${REGION}\\d{4}[A-Z]{1,2}$`) },
  { category: 'bike', regex: new RegExp(`^${REGION}MT\\d{3}[A-Z]{2}$`) },
  { category: 'state', regex: /^(?:CA|AN)\d{4}[A-Z]{1,2}$/ },
  { category: 'diplomatic', regex: /^(?:(?:CMD|CPC|CD|CC|PA)\d{2,3}RC\d{1,4}|CD\d{1,6})$/ },
  { category: 'temporary', regex: /^IT\d{5}RC$/ },
  { category: 'test_vehicle', regex: new RegExp(`^${REGION}\\d{4}WG$`) },
  { category: 'transit', regex: /^WT\d{6,7}$/ },
  { category: 'postal', regex: /^PT\d{5}$/ },
  { category: 'special_investment', regex: /^IS\d{5,6}RC$/ },
  { category: 'national_security', regex: /^SN\d{4}$/ },
  { category: 'military', regex: /^\d{7}$/ },
  { category: 'postal_telecom', regex: /^RT\d{6}$/ },
  { category: 'government_legacy', regex: /^[A-Z]{2}\d{4}[A-Z]$/ },
];

/**
 * Image preprocessing utilities for license plate OCR
 * Optimized for Cameroon plates (orange background, black text)
 */

export class ImageProcessor {
  /**
   * Preprocess image for hybrid OCR
   * Resizes to 800x600 and compresses to JPEG 70% (~50KB)
   * @param imageSrc Base64 image data
   * @returns Processed base64 image
   */
  static async preprocessForOCR(imageSrc: string, t: TFunction): Promise<string> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          if (!ctx) {
            reject(new Error(t('errors.canvasContext')));
            return;
          }

          // Target resolution 800x600
          canvas.width = 800;
          canvas.height = 600;

          // High-quality scaling
          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = 'high';

          // Draw image to fill the 800x600 canvas (may crop slightly if aspect ratio differs)
          const scale = Math.max(800 / img.width, 600 / img.height);
          const x = (800 - img.width * scale) / 2;
          const y = (600 - img.height * scale) / 2;
          ctx.drawImage(img, x, y, img.width * scale, img.height * scale);

          // Return as JPEG with 70% quality for optimal bandwidth (~50KB)
          const result = canvas.toDataURL('image/jpeg', 0.7);

          resolve(result);
        } catch (error) {
          reject(new Error(t('errors.imageProcessing')));
        }
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }

  private static convolve(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    kernel: number[],
    divisor: number,
    bias: number
  ) {
    const src = ctx.getImageData(0, 0, width, height);
    const dst = ctx.createImageData(width, height);
    const s = src.data;
    const d = dst.data;

    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        let r = 0;
        let g = 0;
        let b = 0;
        let a = 0;

        for (let ky = -1; ky <= 1; ky++) {
          const sy = Math.min(height - 1, Math.max(0, y + ky));
          for (let kx = -1; kx <= 1; kx++) {
            const sx = Math.min(width - 1, Math.max(0, x + kx));
            const si = (sy * width + sx) * 4;
            const ki = (ky + 1) * 3 + (kx + 1);
            const w = kernel[ki];
            r += s[si] * w;
            g += s[si + 1] * w;
            b += s[si + 2] * w;
            a += s[si + 3] * w;
          }
        }

        const di = (y * width + x) * 4;
        d[di] = ImageProcessor.clamp(r / divisor + bias);
        d[di + 1] = ImageProcessor.clamp(g / divisor + bias);
        d[di + 2] = ImageProcessor.clamp(b / divisor + bias);
        d[di + 3] = ImageProcessor.clamp(a / divisor + bias);
      }
    }

    ctx.putImageData(dst, 0, 0);
  }

  static async preprocessForPhotoCapture(imageSrc: string, t: TFunction): Promise<string> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          if (!ctx) {
            reject(new Error(t('errors.canvasContext')));
            return;
          }

          const targetWidth = 1800;
          const targetHeight = 900;

          canvas.width = targetWidth;
          canvas.height = targetHeight;

          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = 'high';

          const W = window.innerWidth;
          const H = window.innerHeight;
          const w = img.width;
          const h = img.height;

          const scale = Math.max(W / w, H / h);
          const vw = W * 0.92;
          const vh = vw / 2.0;

          const sw = vw / scale;
          const sh = vh / scale;

          const sx = (w - sw) / 2;
          const sy = (h - sh) / 2;

          ctx.fillStyle = '#FFFFFF';
          ctx.fillRect(0, 0, targetWidth, targetHeight);
          ctx.drawImage(img, sx, sy, sw, sh, 0, 0, targetWidth, targetHeight);

          ImageProcessor.convolve(
            ctx,
            targetWidth,
            targetHeight,
            [0, -1, 0, -1, 5, -1, 0, -1, 0],
            1,
            0
          );

          const result = canvas.toDataURL('image/jpeg', 0.92);
          resolve(result);
        } catch (error) {
          reject(new Error(t('errors.imageProcessing')));
        }
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }

  // ── Viewfinder guide-frame constants ───────────────────────────────────────
  // The viewfinder occupies 92 % of the screen width with a 3.5:1 aspect ratio
  // (accommodates both 520×110 mm standard and 460×135 mm secondary Cameroon plates).
  private static readonly VF_WIDTH_RATIO = 0.92;
  private static readonly VF_ASPECT = 3.5; // width / height

  /**
   * Computes safe crop coordinates (sx, sy, sw, sh) and corresponding
   * destination coordinates (dx, dy, dw, dh) for canvas drawing.
   * Clamps to image boundaries to prevent out-of-bounds drawing,
   * which can occur if window dimensions change or ratios overshoot.
   */
  private static getSafeCrop(imgW: number, imgH: number, targetW: number, targetH: number) {
    const W = window.innerWidth;
    const H = window.innerHeight;

    // The webcam uses object-cover, so the scale is the larger ratio
    const scale = Math.max(W / imgW, H / imgH);

    // Viewfinder dimensions in CSS pixels
    const vw = W * ImageProcessor.VF_WIDTH_RATIO;
    const vh = vw / ImageProcessor.VF_ASPECT;

    // Convert CSS pixels → raw video pixels
    const sw = vw / scale;
    const sh = vh / scale;

    // Center the crop on the video frame
    const sx = (imgW - sw) / 2;
    const sy = (imgH - sh) / 2;

    // Clamp coordinates to image bounds
    const safeSx = Math.max(0, sx);
    const safeSy = Math.max(0, sy);
    const safeSw = Math.min(sw, imgW - safeSx);
    const safeSh = Math.min(sh, imgH - safeSy);

    // Calculate destination coordinates to maintain aspect ratio if clamped
    if (sw === 0 || sh === 0) {
      return { sx: 0, sy: 0, sw: imgW, sh: imgH, dx: 0, dy: 0, dw: targetW, dh: targetH };
    }
    const dx = (targetW - (safeSw / sw) * targetW) / 2;
    const dy = (targetH - (safeSh / sh) * targetH) / 2;
    const dw = (safeSw / sw) * targetW;
    const dh = (safeSh / sh) * targetH;

    return { sx: safeSx, sy: safeSy, sw: safeSw, sh: safeSh, dx, dy, dw, dh };
  }

  /**
   * Preprocess a photo capture for OCR by cropping exactly to the viewfinder
   * guide frame, up-scaling to an OCR-friendly resolution, and applying a
   * light sharpen kernel.
   *
   * This is the PRIMARY photo preprocessing method — it should be called
   * first (not as a fallback).
   */
  static async preprocessForPhoto(imageSrc: string, t: TFunction): Promise<string> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          if (!ctx) {
            reject(new Error(t('errors.canvasContext')));
            return;
          }

          // Output dimensions — 3.5:1 plate-matched aspect ratio
          const targetWidth = 1400;
          const targetHeight = 400; // 1400 / 3.5 = 400

          canvas.width = targetWidth;
          canvas.height = targetHeight;

          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = 'high';

          // Calculate clamped crop and destination coordinates
          const { sx, sy, sw, sh, dx, dy, dw, dh } = ImageProcessor.getSafeCrop(
            img.width,
            img.height,
            targetWidth,
            targetHeight
          );

          // White background (in case crop overshoots)
          ctx.fillStyle = '#FFFFFF';
          ctx.fillRect(0, 0, targetWidth, targetHeight);

          // Draw ONLY the viewfinder region into the output canvas
          ctx.drawImage(img, sx, sy, sw, sh, dx, dy, dw, dh);

          // Light sharpen to compensate for scaling and minor hand tremor
          ImageProcessor.convolve(
            ctx,
            targetWidth,
            targetHeight,
            [0, -1, 0, -1, 6, -1, 0, -1, 0], // mild sharpen (centre = 6 instead of 5)
            2, // divisor — keeps overall brightness stable
            0
          );

          // High quality JPEG to preserve detail for OCR
          const result = canvas.toDataURL('image/jpeg', 0.92);
          resolve(result);
        } catch (error) {
          reject(new Error(t('errors.imageProcessing')));
        }
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }

  /**
   * Lightweight image-quality assessment of the viewfinder region.
   * Returns whether the image is acceptable for OCR and a human-readable
   * feedback string when it is not.
   *
   * Checks:
   *  1. Blur — Laplacian variance (low variance = blurry)
   *  2. Brightness — mean luminance (too dark / too bright)
   */
  static async assessImageQuality(
    imageSrc: string,
    t: TFunction
  ): Promise<{ isAcceptable: boolean; feedback: string }> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          if (!ctx) {
            reject(new Error(t('errors.canvasContext')));
            return;
          }

          // Work on a small version for speed — quality check doesn't need full res
          const checkW = 400;
          const checkH = Math.round(400 / ImageProcessor.VF_ASPECT);
          canvas.width = checkW;
          canvas.height = checkH;

          // Crop to viewfinder region using the shared helper
          const { sx, sy, sw, sh, dx, dy, dw, dh } = ImageProcessor.getSafeCrop(
            img.width,
            img.height,
            checkW,
            checkH
          );

          ctx.drawImage(img, sx, sy, sw, sh, dx, dy, dw, dh);
          const imageData = ctx.getImageData(0, 0, checkW, checkH);
          const pixels = imageData.data;

          // --- Brightness check (mean luminance) ---
          let brightnessSum = 0;
          const pixelCount = checkW * checkH;
          for (let i = 0; i < pixels.length; i += 4) {
            // Fast luminance approximation: (R + G + B) / 3
            brightnessSum += (pixels[i] + pixels[i + 1] + pixels[i + 2]) / 3;
          }
          const meanBrightness = brightnessSum / pixelCount;

          if (meanBrightness < 40) {
            resolve({
              isAcceptable: false,
              feedback: t('mobileScan.qualityTooDark', 'Too dark — move to better lighting'),
            });
            return;
          }
          if (meanBrightness > 220) {
            resolve({
              isAcceptable: false,
              feedback: t('mobileScan.qualityTooBright', 'Too bright — avoid direct sunlight'),
            });
            return;
          }

          // --- Blur check (Laplacian variance) ---
          // Convert to grayscale first
          const gray = new Float32Array(pixelCount);
          for (let i = 0; i < pixelCount; i++) {
            const pi = i * 4;
            gray[i] = 0.299 * pixels[pi] + 0.587 * pixels[pi + 1] + 0.114 * pixels[pi + 2];
          }

          // Apply Laplacian kernel [0,1,0; 1,-4,1; 0,1,0]
          let lapSum = 0;
          let lapSumSq = 0;
          let lapCount = 0;

          for (let y = 1; y < checkH - 1; y++) {
            for (let x = 1; x < checkW - 1; x++) {
              const idx = y * checkW + x;
              const lap =
                gray[idx - checkW] + // top
                gray[idx - 1] + // left
                -4 * gray[idx] + // center
                gray[idx + 1] + // right
                gray[idx + checkW]; // bottom
              lapSum += lap;
              lapSumSq += lap * lap;
              lapCount++;
            }
          }

          const lapMean = lapSum / lapCount;
          const lapVariance = lapSumSq / lapCount - lapMean * lapMean;

          // Threshold determined empirically; typical sharp plate > 200, blurry < 80
          if (lapVariance < 80) {
            resolve({
              isAcceptable: false,
              feedback: t('mobileScan.qualityTooBlurry', 'Image is blurry — hold steady'),
            });
            return;
          }

          resolve({ isAcceptable: true, feedback: '' });
        } catch (error) {
          // If quality check fails, don't block the capture — just allow it
          resolve({ isAcceptable: true, feedback: '' });
        }
      };
      img.onerror = () => resolve({ isAcceptable: true, feedback: '' });
      img.src = imageSrc;
    });
  }

  /**
   * Preprocess image for high-resolution single-shot photo capture.
   * Uses 1600×1200 at JPEG 90% quality for maximum OCR accuracy.
   * @param imageSrc Base64 image data
   * @returns Processed base64 image
   */
  static async preprocessForHighRes(imageSrc: string, t: TFunction): Promise<string> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          if (!ctx) {
            reject(new Error(t('errors.canvasContext')));
            return;
          }

          // High-res target: 1600×1200
          canvas.width = 1600;
          canvas.height = 1200;

          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = 'high';

          // Center-crop to fill canvas
          const scale = Math.max(1600 / img.width, 1200 / img.height);
          const x = (1600 - img.width * scale) / 2;
          const y = (1200 - img.height * scale) / 2;
          ctx.drawImage(img, x, y, img.width * scale, img.height * scale);

          // 90% quality for minimal compression artifacts
          const result = canvas.toDataURL('image/jpeg', 0.9);
          resolve(result);
        } catch (error) {
          reject(new Error(t('errors.imageProcessing')));
        }
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }

  /**
   * Clamp value between 0 and 255
   */
  private static clamp(value: number): number {
    return Math.max(0, Math.min(255, value));
  }

  /**
   * Validate Cameroon license plate format
   * @param text OCR extracted text
   * @returns Cleaned and validated plate number or null
   */
  static validateCameroonPlate(text: string): string | null {
    return ImageProcessor.classifyCameroonPlate(text)?.formatted ?? null;
  }

  /**
   * Classify a Cameroon license plate without changing the legacy validator return type.
   */
  static classifyCameroonPlate(text: string): CameroonPlateClassification | null {
    const plate = ImageProcessor.normalizePlateText(text);
    const match = PLATE_PATTERNS.find(({ regex }) => regex.test(plate));

    if (!match) return null;

    return {
      plate,
      formatted: ImageProcessor.formatCameroonPlate(plate, match.category),
      category: match.category,
    };
  }

  private static normalizePlateText(text: string): string {
    return text.toUpperCase().replace(/[^A-Z0-9]/g, '');
  }

  private static formatCameroonPlate(plate: string, category: string): string {
    switch (category) {
      case 'civil_cemac':
        return `${plate.slice(0, 2)} ${plate.slice(2, 5)} ${plate.slice(5)}`;
      case 'civil_legacy':
      case 'state':
      case 'government_legacy':
        return `${plate.slice(0, 2)} ${plate.slice(2, 6)} ${plate.slice(6)}`;
      case 'trailer':
        return `${plate.slice(0, 2)} ${plate.slice(2, 4)} ${plate.slice(4, 8)} ${plate.slice(8)}`;
      case 'diplomatic': {
        const rcIndex = plate.indexOf('RC');
        if (rcIndex > 0) {
          const prefix =
            plate.startsWith('CMD') || plate.startsWith('CPC')
              ? plate.slice(0, 3)
              : plate.slice(0, 2);
          return `${prefix} ${plate.slice(prefix.length, rcIndex)} RC ${plate.slice(rcIndex + 2)}`;
        }
        return `${plate.slice(0, 2)} ${plate.slice(2)}`;
      }
      case 'temporary':
        return `IT ${plate.slice(2, 7)} RC`;
      case 'test_vehicle':
        return `${plate.slice(0, 2)} ${plate.slice(2, 6)} WG`;
      case 'transit':
        return `WT ${plate.slice(2)}`;
      case 'postal':
        return `PT ${plate.slice(2)}`;
      case 'special_investment':
        return `IS ${plate.slice(2, -2)} RC`;
      case 'national_security':
        return `SN ${plate.slice(2)}`;
      case 'postal_telecom':
        return `RT ${plate.slice(2)}`;
      default:
        return plate;
    }
  }

  /**
   * Scale image to optimal size for OCR
   * Tesseract works best with images around 300 DPI
   */
  static async scaleImage(
    imageSrc: string,
    t: TFunction,
    targetWidth: number = 800
  ): Promise<string> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');

        if (!ctx) {
          reject(new Error(t('errors.canvasContext')));
          return;
        }

        // Calculate scaled dimensions
        const scale = targetWidth / img.width;
        canvas.width = targetWidth;
        canvas.height = img.height * scale;

        // Use high-quality scaling
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = 'high';

        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
        resolve(canvas.toDataURL('image/jpeg', 0.95));
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }

  /**
   * Crop image to a center box for better OCR accuracy
   * Viewfinder is roughly 3:1 aspect ratio in the center
   */
  static async cropToViewfinder(imageSrc: string, t: TFunction): Promise<string> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          if (!ctx) {
            reject(new Error(t('errors.canvasContext')));
            return;
          }

          // Output dimensions — 3.5:1 aspect ratio matching Cameroon plates
          const targetWidth = 1200;
          const targetHeight = Math.round(1200 / ImageProcessor.VF_ASPECT);

          canvas.width = targetWidth;
          canvas.height = targetHeight;

          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = 'high';

          // Calculate clamped crop and destination coordinates
          const { sx, sy, sw, sh, dx, dy, dw, dh } = ImageProcessor.getSafeCrop(
            img.width,
            img.height,
            targetWidth,
            targetHeight
          );

          // White background
          ctx.fillStyle = '#FFFFFF';
          ctx.fillRect(0, 0, targetWidth, targetHeight);

          // Draw without squashing!
          ctx.drawImage(img, sx, sy, sw, sh, dx, dy, dw, dh);

          resolve(canvas.toDataURL('image/jpeg', 0.9));
        } catch (error) {
          reject(new Error(t('errors.imageProcessing')));
        }
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }

  static async cropToViewfinderFast(imageSrc: string, t: TFunction): Promise<string> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');

          if (!ctx) {
            reject(new Error(t('errors.canvasContext')));
            return;
          }

          // Smaller output for live scanning to reduce upload + backend time
          const targetWidth = 800;
          const targetHeight = Math.round(800 / ImageProcessor.VF_ASPECT);

          canvas.width = targetWidth;
          canvas.height = targetHeight;

          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = 'high';

          const { sx, sy, sw, sh, dx, dy, dw, dh } = ImageProcessor.getSafeCrop(
            img.width,
            img.height,
            targetWidth,
            targetHeight
          );

          ctx.fillStyle = '#FFFFFF';
          ctx.fillRect(0, 0, targetWidth, targetHeight);
          ctx.drawImage(img, sx, sy, sw, sh, dx, dy, dw, dh);

          // Lower quality for speed in live mode
          resolve(canvas.toDataURL('image/jpeg', 0.65));
        } catch (error) {
          reject(new Error(t('errors.imageProcessing')));
        }
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }
}
