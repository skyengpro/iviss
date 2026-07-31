import { useEffect } from 'react';

const COACHING_INTERVAL_MS = 250; // ~4Hz

interface UseCaptureCoachingProps {
  mode: 'photo' | 'live';
  photoState: 'idle' | 'processing' | 'result' | 'error';
  isScanning: boolean;
  getPreviewScreenshot: () => string | null;
  onFrame?: (previewDataUrl: string) => void;
}

/**
 * Concurrency-safe scaffold for real-time framing coaching. Deliberately
 * gated to `mode === 'photo' && photoState === 'idle' && !isScanning` from
 * the first line — a prior iteration ran this loop unconditionally during
 * live scanning, doubling CPU load on the phone with a full-frame encode
 * for a measurement that renormalizes to a fixed probe width anyway.
 * Frame analysis and UI feedback are not implemented here — `onFrame`
 * receives the capped preview for a future coaching UI to consume.
 */
export function useCaptureCoaching({
  mode,
  photoState,
  isScanning,
  getPreviewScreenshot,
  onFrame,
}: UseCaptureCoachingProps): void {
  const active = mode === 'photo' && photoState === 'idle' && !isScanning;

  useEffect(() => {
    if (!active || !onFrame) return;

    const interval = setInterval(() => {
      const preview = getPreviewScreenshot();
      if (preview) onFrame(preview);
    }, COACHING_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [active, getPreviewScreenshot, onFrame]);
}
