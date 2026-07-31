import React from 'react';
import Webcam from 'react-webcam';
import { VF_ASPECT, VF_WIDTH_RATIO } from '@/utils/viewfinder';

interface ScanViewfinderProps {
  webcamRef: React.RefObject<Webcam>;
  facingMode: 'user' | 'environment';
  isScanning: boolean;
  mode: 'photo' | 'live';
  liveScanActive: boolean;
  capturedImageSrc?: string | null;
  hasError?: boolean;
  onUserMedia?: () => void;
  onUserMediaError?: (error: string | DOMException) => void;
}

export const ScanViewfinder: React.FC<ScanViewfinderProps> = ({
  webcamRef,
  facingMode,
  isScanning,
  mode,
  liveScanActive,
  capturedImageSrc,
  hasError,
  onUserMedia,
  onUserMediaError,
}) => {
  const videoConstraints = {
    facingMode: facingMode,
    width: { ideal: 1920 },
    height: { ideal: 1080 },
  };

  const borderColor = hasError ? 'border-destructive' : 'border-accent';

  return (
    <div className="relative flex-1 bg-black">
      {mode === 'photo' && capturedImageSrc ? (
        <img
          src={capturedImageSrc}
          alt="Captured"
          className="absolute inset-0 h-full w-full object-cover"
        />
      ) : (
        <Webcam
          audio={false}
          ref={webcamRef}
          screenshotFormat="image/jpeg"
          videoConstraints={videoConstraints}
          forceScreenshotSourceSize
          className="absolute inset-0 h-full w-full object-cover"
          onUserMedia={onUserMedia}
          onUserMediaError={onUserMediaError}
          mirrored={facingMode === 'user'}
        />
      )}

      {/* Scan frame overlay — width/aspect come from utils/viewfinder.ts, the same
          VF_WIDTH_RATIO/VF_ASPECT source computeViewfinderCrop uses for the crop
          sent to OCR, so this frame and the actual crop stay in sync by construction.
          No independent max-width cap: that used to diverge from the crop math on
          larger screens (see ocr_perf_improvement/02_ticket_frontend.md §3). */}
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
        <div
          data-testid="viewfinder-frame"
          className="relative"
          style={{ width: `${VF_WIDTH_RATIO * 100}%`, aspectRatio: VF_ASPECT }}
        >
          {/* Corner markers */}
          <div
            className={`absolute left-0 top-0 h-5 w-8 border-l-4 border-t-4 transition-colors duration-300 ${borderColor}`}
          />
          <div
            className={`absolute right-0 top-0 h-5 w-8 border-r-4 border-t-4 transition-colors duration-300 ${borderColor}`}
          />
          <div
            className={`absolute bottom-0 left-0 h-5 w-8 border-b-4 border-l-4 transition-colors duration-300 ${borderColor}`}
          />
          <div
            className={`absolute bottom-0 right-0 h-5 w-8 border-b-4 border-r-4 transition-colors duration-300 ${borderColor}`}
          />

          {/* Scan line animation for live mode */}
          {mode === 'live' && liveScanActive && (
            <div className="absolute inset-0 overflow-hidden">
              <div className="absolute inset-x-0 h-1 bg-gradient-to-r from-transparent via-accent to-transparent animate-scan-line" />
            </div>
          )}

          {/* Scanning indicator */}
          {isScanning && (
            <div className="absolute -bottom-12 left-1/2 -translate-x-1/2 flex items-center gap-2 rounded-full bg-black/70 px-4 py-2 text-white text-sm border border-white/20 backdrop-blur-sm">
              <div className="h-4 w-4 animate-spin rounded-full border-2 border-accent border-t-transparent" />
              <span>Scanning…</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
