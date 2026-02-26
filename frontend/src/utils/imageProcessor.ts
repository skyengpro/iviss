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

          // Output dimensions — wide strip for a plate
          const targetWidth = 900;
          const targetHeight = 120;

          canvas.width = targetWidth;
          canvas.height = targetHeight;
          
          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = 'high';

          // Match the on-screen viewfinder: aspect-[3/1], ~80% of screen width, centered
          // The webcam is stretched to fill the screen (object-cover), so
          // we crop the same proportional region from the camera frame.
          const sourceWidth = img.width * 0.80;
          const sourceHeight = sourceWidth / 3; // 3:1 aspect ratio like the viewfinder

          const sx = (img.width - sourceWidth) / 2;
          const sy = (img.height - sourceHeight) / 2;

          // White background (helps Tesseract with edge chars)
          ctx.fillStyle = '#FFFFFF';
          ctx.fillRect(0, 0, targetWidth, targetHeight);

          ctx.drawImage(img, sx, sy, sourceWidth, sourceHeight, 0, 0, targetWidth, targetHeight);

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
