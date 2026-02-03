/**
 * Image preprocessing utilities for license plate OCR
 * Optimized for Cameroon plates (orange background, black text)
 */

export class ImageProcessor {
    /**
   * Preprocess image for better OCR accuracy (SIMPLIFIED VERSION)
   * @param imageSrc Base64 image data
   * @returns Processed base64 image
   */
    static async preprocessForOCR(imageSrc: string): Promise<string> {
        return new Promise((resolve, reject) => {
            const img = new Image();
            img.onload = () => {
                try {
                    const canvas = document.createElement('canvas');
                    const ctx = canvas.getContext('2d');

                    if (!ctx) {
                        reject(new Error('Could not get canvas context'));
                        return;
                    }

                    // Ensure reasonable size (not too small, not too large)
                    const targetWidth = 800;
                    const scale = targetWidth / img.width;
                    const targetHeight = Math.floor(img.height * scale);

                    canvas.width = targetWidth;
                    canvas.height = targetHeight;

                    // High-quality scaling
                    ctx.imageSmoothingEnabled = true;
                    ctx.imageSmoothingQuality = 'high';
                    ctx.drawImage(img, 0, 0, targetWidth, targetHeight);

                    // Get image data
                    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
                    const data = imageData.data;

                    // MINIMAL preprocessing: just grayscale and slight contrast boost
                    this.convertToGrayscale(data);
                    this.enhanceContrast(data, 1.3); // Gentle contrast enhancement

                    // Put processed data back
                    ctx.putImageData(imageData, 0, 0);

                    // Return as base64
                    const result = canvas.toDataURL('image/jpeg', 0.95);

                    resolve(result);
                } catch (error) {
                    reject(error);
                }
            };
            img.onerror = () => reject(new Error('Failed to load image'));
            img.src = imageSrc;
        });
    }

    /**
     * Convert image to grayscale
     */
    private static convertToGrayscale(data: Uint8ClampedArray): void {
        for (let i = 0; i < data.length; i += 4) {
            const gray = 0.299 * data[i] + 0.587 * data[i + 1] + 0.114 * data[i + 2];
            data[i] = gray;     // R
            data[i + 1] = gray; // G
            data[i + 2] = gray; // B
            // Alpha channel (i+3) unchanged
        }
    }

    /**
   * Enhance contrast - particularly useful for orange plates
   */
    private static enhanceContrast(data: Uint8ClampedArray, factor: number = 2.0): void {
        const contrast = (factor - 1) * 128;

        for (let i = 0; i < data.length; i += 4) {
            data[i] = this.clamp(factor * data[i] + contrast);
            data[i + 1] = this.clamp(factor * data[i + 1] + contrast);
            data[i + 2] = this.clamp(factor * data[i + 2] + contrast);
        }
    }

    /**
     * Apply adaptive thresholding for better text extraction
     */
    private static applyAdaptiveThreshold(
        data: Uint8ClampedArray,
        width: number,
        height: number,
        blockSize: number = 15
    ): void {
        // Use Otsu's method for automatic threshold calculation
        const histogram = new Array(256).fill(0);

        // Build histogram
        for (let i = 0; i < data.length; i += 4) {
            histogram[data[i]]++;
        }

        // Calculate threshold using Otsu's method
        const total = width * height;
        let sum = 0;
        for (let i = 0; i < 256; i++) {
            sum += i * histogram[i];
        }

        let sumB = 0;
        let wB = 0;
        let wF = 0;
        let maxVariance = 0;
        let threshold = 0;

        for (let i = 0; i < 256; i++) {
            wB += histogram[i];
            if (wB === 0) continue;

            wF = total - wB;
            if (wF === 0) break;

            sumB += i * histogram[i];
            const mB = sumB / wB;
            const mF = (sum - sumB) / wF;
            const variance = wB * wF * (mB - mF) * (mB - mF);

            if (variance > maxVariance) {
                maxVariance = variance;
                threshold = i;
            }
        }

        // Apply threshold
        for (let i = 0; i < data.length; i += 4) {
            const value = data[i];
            const binary = value > threshold ? 255 : 0;

            data[i] = binary;
            data[i + 1] = binary;
            data[i + 2] = binary;
        }
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
    static async scaleImage(imageSrc: string, targetWidth: number = 800): Promise<string> {
        return new Promise((resolve, reject) => {
            const img = new Image();
            img.onload = () => {
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');

                if (!ctx) {
                    reject(new Error('Could not get canvas context'));
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
            img.onerror = () => reject(new Error('Failed to load image'));
            img.src = imageSrc;
        });
    }
}
