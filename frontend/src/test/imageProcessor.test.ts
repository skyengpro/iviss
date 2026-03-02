import { describe, it, expect } from 'vitest';

import { ImageProcessor } from '@/utils/imageProcessor';

describe('ImageProcessor.validateCameroonPlate', () => {
  it('should format a valid plate as "XX ### XX"', () => {
    expect(ImageProcessor.validateCameroonPlate('CE128BC')).toBe('CE 128 BC');
    expect(ImageProcessor.validateCameroonPlate('CE 128 BC')).toBe('CE 128 BC');
    expect(ImageProcessor.validateCameroonPlate('CE-128-BC')).toBe('CE 128 BC');
  });

  it('should return null for invalid plates', () => {
    expect(ImageProcessor.validateCameroonPlate('')).toBeNull();
    expect(ImageProcessor.validateCameroonPlate('CE12BC')).toBeNull();
    expect(ImageProcessor.validateCameroonPlate('CE12BCA')).toBeNull();
    expect(ImageProcessor.validateCameroonPlate('1234567')).toBeNull();
  });
});
