import { describe, it, expect } from 'vitest';
import { cn } from '../utils';

describe('cn utility', () => {
  it('should merge multiple class names', () => {
    const result = cn('text-red-500', 'bg-blue-500');
    expect(result).toBe('text-red-500 bg-blue-500');
  });

  it('should handle conditional classes via clsx', () => {
    const isActive = true;
    const isDisabled = false;

    const result = cn('base-class', isActive && 'active-class', isDisabled && 'disabled-class');

    expect(result).toContain('base-class');
    expect(result).toContain('active-class');
    expect(result).not.toContain('disabled-class');
  });

  it('should deduplicate conflicting Tailwind classes via tailwind-merge', () => {
    // tailwind-merge should keep only the last conflicting class
    const result = cn('px-4', 'px-8');
    expect(result).toBe('px-8');
  });

  it('should handle undefined and null values gracefully', () => {
    const result = cn('base', undefined, null, 'extra');
    expect(result).toBe('base extra');
  });

  it('should return empty string when called with no arguments', () => {
    const result = cn();
    expect(result).toBe('');
  });
});
