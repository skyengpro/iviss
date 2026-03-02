import React from 'react';
import Webcam from 'react-webcam';
import { cn } from '@/lib/utils';

interface ScanViewfinderProps {
  webcamRef: React.RefObject<Webcam>;
  facingMode: 'user' | 'environment';
  isScanning: boolean;
  mode: 'photo' | 'live';
  liveScanActive: boolean;
  onUserMedia?: () => void;
  onUserMediaError?: (error: string | DOMException) => void;
}

export const ScanViewfinder: React.FC<ScanViewfinderProps> = ({
  webcamRef,
  facingMode,
  isScanning,
  mode,
  liveScanActive,
  onUserMedia,
  onUserMediaError,
}) => {
  const videoConstraints = {
    facingMode: facingMode,
  };

  return (
    <div className="relative flex-1 bg-black">
      <Webcam
        audio={false}
        ref={webcamRef}
        screenshotFormat="image/jpeg"
        videoConstraints={videoConstraints}
        className="absolute inset-0 h-full w-full object-cover"
        onUserMedia={onUserMedia}
        onUserMediaError={onUserMediaError}
        mirrored={facingMode === 'user'}
      />

      {/* Scan frame overlay */}
      <div className="absolute inset-0 flex items-center justify-center p-8 pointer-events-none">
        <div className="relative aspect-[3/1] w-full max-w-sm">
          {/* Corner markers */}
          <div className="absolute left-0 top-0 h-8 w-8 border-l-4 border-t-4 border-accent" />
          <div className="absolute right-0 top-0 h-8 w-8 border-r-4 border-t-4 border-accent" />
          <div className="absolute bottom-0 left-0 h-8 w-8 border-b-4 border-l-4 border-accent" />
          <div className="absolute bottom-0 right-0 h-8 w-8 border-b-4 border-r-4 border-accent" />

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
