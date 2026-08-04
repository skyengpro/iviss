import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useCaptureCoaching } from '../feature/useCaptureCoaching';

type Mode = 'photo' | 'live';
type PhotoState = 'idle' | 'processing' | 'result' | 'error';

function setup(props: {
  mode: Mode;
  photoState: PhotoState;
  isScanning: boolean;
  getPreviewScreenshot: () => string | null;
  onFrame?: (preview: string) => void;
}) {
  return renderHook((p) => useCaptureCoaching(p), { initialProps: props });
}

describe('useCaptureCoaching', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('does not sample while in live mode', async () => {
    const getPreviewScreenshot = vi.fn(() => 'data:image/jpeg;base64,preview');
    const onFrame = vi.fn();

    setup({ mode: 'live', photoState: 'idle', isScanning: false, getPreviewScreenshot, onFrame });

    await vi.advanceTimersByTimeAsync(1000);

    expect(getPreviewScreenshot).not.toHaveBeenCalled();
    expect(onFrame).not.toHaveBeenCalled();
  });

  it('does not sample while a photo is processing/showing a result', async () => {
    const getPreviewScreenshot = vi.fn(() => 'data:image/jpeg;base64,preview');
    const onFrame = vi.fn();

    setup({
      mode: 'photo',
      photoState: 'processing',
      isScanning: false,
      getPreviewScreenshot,
      onFrame,
    });

    await vi.advanceTimersByTimeAsync(1000);

    expect(getPreviewScreenshot).not.toHaveBeenCalled();
  });

  it('does not sample while isScanning is true — the concurrency trap from ticket §6', async () => {
    const getPreviewScreenshot = vi.fn(() => 'data:image/jpeg;base64,preview');
    const onFrame = vi.fn();

    setup({ mode: 'photo', photoState: 'idle', isScanning: true, getPreviewScreenshot, onFrame });

    await vi.advanceTimersByTimeAsync(1000);

    expect(getPreviewScreenshot).not.toHaveBeenCalled();
  });

  it('samples the capped preview at ~4Hz once mode=photo, photoState=idle and not scanning', async () => {
    const getPreviewScreenshot = vi.fn(() => 'data:image/jpeg;base64,preview');
    const onFrame = vi.fn();

    setup({ mode: 'photo', photoState: 'idle', isScanning: false, getPreviewScreenshot, onFrame });

    await vi.advanceTimersByTimeAsync(1000);

    expect(getPreviewScreenshot).toHaveBeenCalledTimes(4);
    expect(onFrame).toHaveBeenCalledTimes(4);
    expect(onFrame).toHaveBeenCalledWith('data:image/jpeg;base64,preview');
  });

  it('does not call onFrame when the preview is unavailable', async () => {
    const getPreviewScreenshot = vi.fn(() => null);
    const onFrame = vi.fn();

    setup({ mode: 'photo', photoState: 'idle', isScanning: false, getPreviewScreenshot, onFrame });

    await vi.advanceTimersByTimeAsync(1000);

    expect(getPreviewScreenshot).toHaveBeenCalled();
    expect(onFrame).not.toHaveBeenCalled();
  });

  it('does not start a sampling loop at all when no onFrame consumer is given', async () => {
    const getPreviewScreenshot = vi.fn(() => 'data:image/jpeg;base64,preview');

    setup({ mode: 'photo', photoState: 'idle', isScanning: false, getPreviewScreenshot });

    await vi.advanceTimersByTimeAsync(1000);

    expect(getPreviewScreenshot).not.toHaveBeenCalled();
  });

  it('stops sampling as soon as the gate turns off, e.g. isScanning becomes true', async () => {
    const getPreviewScreenshot = vi.fn(() => 'data:image/jpeg;base64,preview');
    const onFrame = vi.fn();

    const { rerender } = setup({
      mode: 'photo',
      photoState: 'idle',
      isScanning: false,
      getPreviewScreenshot,
      onFrame,
    });

    await vi.advanceTimersByTimeAsync(500);
    expect(getPreviewScreenshot).toHaveBeenCalledTimes(2);

    rerender({
      mode: 'photo',
      photoState: 'idle',
      isScanning: true,
      getPreviewScreenshot,
      onFrame,
    });

    await vi.advanceTimersByTimeAsync(1000);
    expect(getPreviewScreenshot).toHaveBeenCalledTimes(2);
  });

  it('clears the interval on unmount', async () => {
    const getPreviewScreenshot = vi.fn(() => 'data:image/jpeg;base64,preview');
    const onFrame = vi.fn();

    const { unmount } = setup({
      mode: 'photo',
      photoState: 'idle',
      isScanning: false,
      getPreviewScreenshot,
      onFrame,
    });

    await vi.advanceTimersByTimeAsync(250);
    expect(getPreviewScreenshot).toHaveBeenCalledTimes(1);

    unmount();

    await vi.advanceTimersByTimeAsync(1000);
    expect(getPreviewScreenshot).toHaveBeenCalledTimes(1);
  });
});
