import { describe, it, expect } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useAuth } from '../auth/use-auth';

describe('useAuth', () => {
  it('should throw when called outside of AuthProvider', () => {
    // renderHook will throw because there is no AuthProvider wrapping it
    expect(() => {
      renderHook(() => useAuth());
    }).toThrow('useAuth must be used within an AuthProvider');
  });
});
