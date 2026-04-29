import { describe, it, expect } from 'vitest';
import { isValidPlate } from '../PlateInput';

describe('PlateInput Validation', () => {
  it('should validate Standard Regional format (REGION 1234 A)', () => {
    expect(isValidPlate('CE 1234 A')).toBe(true);
    expect(isValidPlate('LT 1234 B')).toBe(true);
    expect(isValidPlate('AD 0001 Z')).toBe(true);
  });

  it('should validate Standard Regional format (REGION 123 AB)', () => {
    expect(isValidPlate('LT 123 AB')).toBe(true);
    expect(isValidPlate('CE 999 ZZ')).toBe(true);
  });

  it('should validate Police format (SN 1234)', () => {
    expect(isValidPlate('SN 1234')).toBe(true);
  });

  it('should validate Military format (7 digits)', () => {
    expect(isValidPlate('1234567')).toBe(true);
    expect(isValidPlate('9999999')).toBe(true);
  });

  it('should validate Government format (EN1234X)', () => {
    expect(isValidPlate('EN1234X')).toBe(true);
    expect(isValidPlate('CA1234Y')).toBe(true);
  });

  it('should validate Postal format (RT123456)', () => {
    expect(isValidPlate('RT123456')).toBe(true);
  });

  it('should validate Diplomatic format (CD 12 345)', () => {
    expect(isValidPlate('CD 01 123')).toBe(true);
    expect(isValidPlate('CD 123 456')).toBe(true);
  });

  it('should handle trailing spaces gracefully', () => {
    expect(isValidPlate('CE 1234 A ')).toBe(true);
    expect(isValidPlate(' SN 1234')).toBe(true);
  });

  it('should reject invalid formats', () => {
    expect(isValidPlate('INVALID')).toBe(false);
    expect(isValidPlate('AB 12345 C')).toBe(false);
    expect(isValidPlate('123456')).toBe(false); // Too short for military
    expect(isValidPlate('12345678')).toBe(false); // Too long for military
    expect(isValidPlate('SN1234')).toBe(false); // Missing space (based on regex)
  });
});
