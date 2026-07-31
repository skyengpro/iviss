import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/react';
import { createRef } from 'react';
import Webcam from 'react-webcam';
import { ScanViewfinder } from '../ScanViewfinder';
import { VF_ASPECT, VF_WIDTH_RATIO } from '@/utils/viewfinder';

vi.mock('react-webcam', () => ({
  default: vi.fn(() => null),
}));

function renderViewfinder(overrides: Record<string, unknown> = {}) {
  const webcamRef = createRef<Webcam>();
  return render(
    <ScanViewfinder
      webcamRef={webcamRef}
      facingMode="environment"
      isScanning={false}
      mode="live"
      liveScanActive={false}
      {...overrides}
    />
  );
}

describe('ScanViewfinder', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // React warns about refs on a plain mocked function component — harmless here.
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
  });

  it('sizes the overlay frame from the shared VF_ASPECT/VF_WIDTH_RATIO single source of truth', () => {
    const { getByTestId } = renderViewfinder();
    const frame = getByTestId('viewfinder-frame');

    // Same constants (and no independent max-width cap) as computeViewfinderCrop
    // uses for the crop actually sent to OCR — this is what keeps the two in sync.
    expect(frame.style.aspectRatio).toBe(String(VF_ASPECT));
    expect(frame.style.width).toBe(`${VF_WIDTH_RATIO * 100}%`);
  });

  it('requests native 1920x1080 capture and forces the screenshot to match it', () => {
    renderViewfinder();

    expect(Webcam).toHaveBeenCalledWith(
      expect.objectContaining({
        forceScreenshotSourceSize: true,
        videoConstraints: expect.objectContaining({
          facingMode: 'environment',
          width: { ideal: 1920 },
          height: { ideal: 1080 },
        }),
      }),
      expect.anything()
    );
  });

  it('shows the captured photo instead of the live feed once available', () => {
    const { getByAltText } = renderViewfinder({
      mode: 'photo',
      capturedImageSrc: 'data:image/jpeg;base64,frozen',
    });

    expect(getByAltText('Captured')).toHaveAttribute('src', 'data:image/jpeg;base64,frozen');
    expect(Webcam).not.toHaveBeenCalled();
  });

  it('switches the corner marker color when hasError is set', () => {
    const { container, rerender } = renderViewfinder({ hasError: false });
    expect(container.querySelector('.border-accent')).not.toBeNull();

    rerender(
      <ScanViewfinder
        webcamRef={createRef<Webcam>()}
        facingMode="environment"
        isScanning={false}
        mode="live"
        liveScanActive={false}
        hasError
      />
    );
    expect(container.querySelector('.border-destructive')).not.toBeNull();
  });
});
