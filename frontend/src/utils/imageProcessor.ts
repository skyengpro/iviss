 import { TFunction } from 'i18next';

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
   * Format: XX ### XX (e.g., "CE 128 BC", "LT 390 HN")
   * @param text OCR extracted text
   * @returns Cleaned and validated plate number or null
   */
  static validateCameroonPlate(text: string): string | null {
    // Remove all whitespace and special characters except letters and numbers
    const cleaned = text.replace(/[^A-Z0-9]/g, '');

    // Cameroon plate format: 2 letters + 3 digits + 2 letters
    const regex = /^([A-Z]{2})(\d{3})([A-Z]{2})$/;
    const match = cleaned.match(regex);

    if (match) {
      // Format as: XX ### XX
      return `${match[1]} ${match[2]} ${match[3]}`;
    }

    return null;
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

          // Output dimensions — 2:1 aspect ratio for a plate (supports 2-line/stacked)
          const targetWidth = 1200; 
          const targetHeight = 600;

          canvas.width = targetWidth;
          canvas.height = targetHeight;
          
          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = 'high';

          // 1. Account for 'object-cover' scaling
          const W = window.innerWidth;
          const H = window.innerHeight;
          const w = img.width;
          const h = img.height;

          // object-cover scale
          const scale = Math.max(W / w, H / h);
          
          // Viewfinder-mapped region: 92% of screen width, 2:1 aspect ratio
          const vw = W * 0.92; 
          const vh = vw / 2.0; 

          // Map viewfinder pixels back to raw video pixels
          const sw = vw / scale;
          const sh = vh / scale;

          // Center the crop on the video
          const sx = (w - sw) / 2;
          const sy = (h - sh) / 2;

          // White background
          ctx.fillStyle = '#FFFFFF';
          ctx.fillRect(0, 0, targetWidth, targetHeight);

          // Draw without squashing! (sw/sh aspect == targetWidth/targetHeight aspect)
          ctx.drawImage(img, sx, sy, sw, sh, 0, 0, targetWidth, targetHeight);

          resolve(canvas.toDataURL('image/jpeg', 0.9));
        } catch (error) {
          reject(new Error(t('errors.imageProcessing')));
        }
      };
      img.onerror = () => reject(new Error(t('errors.imageLoad')));
      img.src = imageSrc;
    });
  }
}
