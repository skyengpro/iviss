import { describe, it, expect } from 'vitest';
import { reducer } from '../ui/use-toast';

// Helper to create a toast-like object for testing
function makeToast(id: string, open = true) {
  return { id, open, onOpenChange: () => {} } as any;
}

describe('use-toast reducer', () => {
  describe('ADD_TOAST', () => {
    it('should add a toast to an empty list', () => {
      const state = { toasts: [] };
      const toast = makeToast('1');
      const result = reducer(state, { type: 'ADD_TOAST', toast });

      expect(result.toasts).toHaveLength(1);
      expect(result.toasts[0].id).toBe('1');
    });

    it('should respect TOAST_LIMIT of 1 by keeping only the newest', () => {
      const state = { toasts: [makeToast('old')] };
      const newToast = makeToast('new');
      const result = reducer(state, { type: 'ADD_TOAST', toast: newToast });

      expect(result.toasts).toHaveLength(1);
      expect(result.toasts[0].id).toBe('new');
    });
  });

  describe('UPDATE_TOAST', () => {
    it('should merge partial props into the matching toast', () => {
      const state = { toasts: [makeToast('1')] };
      const result = reducer(state, {
        type: 'UPDATE_TOAST',
        toast: { id: '1', title: 'Updated Title' } as any,
      });

      expect(result.toasts[0].title).toBe('Updated Title');
      expect(result.toasts[0].open).toBe(true); // original prop preserved
    });

    it('should not modify toasts with different IDs', () => {
      const state = { toasts: [makeToast('1'), makeToast('2')] };
      const result = reducer(state, {
        type: 'UPDATE_TOAST',
        toast: { id: '1', title: 'Only for 1' } as any,
      });

      expect((result.toasts[1] as any).title).toBeUndefined();
    });
  });

  describe('DISMISS_TOAST', () => {
    it('should set open to false for the target toast', () => {
      const state = { toasts: [makeToast('1', true)] };
      const result = reducer(state, { type: 'DISMISS_TOAST', toastId: '1' });

      expect(result.toasts[0].open).toBe(false);
    });

    it('should dismiss all toasts when no toastId is provided', () => {
      const state = { toasts: [makeToast('1', true)] };
      const result = reducer(state, { type: 'DISMISS_TOAST' });

      expect(result.toasts.every((t: any) => t.open === false)).toBe(true);
    });
  });

  describe('REMOVE_TOAST', () => {
    it('should remove a specific toast by ID', () => {
      const state = { toasts: [makeToast('1'), makeToast('2')] };
      const result = reducer(state, { type: 'REMOVE_TOAST', toastId: '1' });

      expect(result.toasts).toHaveLength(1);
      expect(result.toasts[0].id).toBe('2');
    });

    it('should clear all toasts when no toastId is provided', () => {
      const state = { toasts: [makeToast('1'), makeToast('2')] };
      const result = reducer(state, { type: 'REMOVE_TOAST' });

      expect(result.toasts).toHaveLength(0);
    });
  });
});
