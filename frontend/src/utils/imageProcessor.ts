import { TFunction } from 'i18next';
import { computeViewfinderCrop, VF_ASPECT } from '@/utils/viewfinder';

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
  private static readonly PHOTO_JPEG_QUALITY = 0.95;
  private static readonly PRIMARY_PHOTO_MAX_WIDTH = 1400;
  private static readonly FALLBACK_PHOTO_MAX_WIDTH = 1800;
  private static readonly LIVE_CROP_OPTIONS = { maxWidth: 800, quality: 0.95 };

  /**
   * Crops the viewfinder region out of `img` and draws it into `canvas` at
   * native resolution, downscaled to `maxWidth` only if wider — never
   * upscaled. Output aspect always matches the crop itself, so a degenerate
   * crop (fallback to full image) never gets squashed into a fixed box.
   */
  private static drawViewfinderCrop(
    img: HTMLImageElement,
    canvas: HTMLCanvasElement,
    ctx: CanvasRenderingContext2D,
    maxWidth: number
  ): void {
    const crop = computeViewfinderCrop(
      img.width,
      img.height,
      window.innerWidth,
      window.innerHeight
    ) ?? { sx: 0, sy: 0, sw: img.width, sh: img.height };

    const outWidth = Math.min(crop.sw, maxWidth);
    const outHeight = outWidth * (crop.sh / crop.sw);

    canvas.width = Math.round(outWidth);
    canvas.height = Math.round(outHeight);

    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(img, crop.sx, crop.sy, crop.sw, crop.sh, 0, 0, canvas.width, canvas.height);
  }

  /**
   * Preprocess a photo capture for OCR by cropping exactly to the viewfinder
   * guide frame, at native resolution (downscale-only, never upscale).
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

          ImageProcessor.drawViewfinderCrop(
            img,
            canvas,
            ctx,
            ImageProcessor.PRIMARY_PHOTO_MAX_WIDTH
          );

          const result = canvas.toDataURL('image/jpeg', ImageProcessor.PHOTO_JPEG_QUALITY);
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
   * Fallback photo preprocessing, used only when the primary attempt
   * returns nothing usable. Same viewfinder-crop geometry as the primary
   * path, at a slightly larger downscale ceiling.
   */
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

          ImageProcessor.drawViewfinderCrop(
            img,
            canvas,
            ctx,
            ImageProcessor.FALLBACK_PHOTO_MAX_WIDTH
          );

          resolve(canvas.toDataURL('image/jpeg', ImageProcessor.PHOTO_JPEG_QUALITY));
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
          const checkH = Math.round(400 / VF_ASPECT);
          canvas.width = checkW;
          canvas.height = checkH;

          const crop = computeViewfinderCrop(
            img.width,
            img.height,
            window.innerWidth,
            window.innerHeight
          ) ?? { sx: 0, sy: 0, sw: img.width, sh: img.height };

          ctx.drawImage(img, crop.sx, crop.sy, crop.sw, crop.sh, 0, 0, checkW, checkH);
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

          ImageProcessor.drawViewfinderCrop(
            img,
            canvas,
            ctx,
            ImageProcessor.LIVE_CROP_OPTIONS.maxWidth
          );

          resolve(canvas.toDataURL('image/jpeg', ImageProcessor.LIVE_CROP_OPTIONS.quality));
        } catch (error) {
          reject(new Error(t('errors.imageProcessing')));
        }
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }
}
