import { useState, useCallback, useEffect } from 'react';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useScanPlate, DetectedPlate } from '@/hooks/feature/useScanPlate';
import { usePhotoCapture } from '@/hooks/feature/usePhotoCapture';
import { useCamera } from '@/hooks/feature/useCamera';
import { ScanViewfinder } from '@/components/mobile/scan/ScanViewfinder';
import { ScanTopControls } from '@/components/mobile/scan/ScanTopControls';
import { ScanActionButtons } from '@/components/mobile/scan/ScanActionButtons';
import { ScanResultCard } from '@/components/mobile/scan/ScanResultCard';
import { Button } from '@/components/ui/button';
import { Keyboard } from 'lucide-react';

type ScanMode = 'photo' | 'live';

export default function MobileScan() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { t } = useTranslation();
  const initialMode = (searchParams.get('mode') as ScanMode) || 'photo';

  const [mode, setMode] = useState<ScanMode>(initialMode);
  const [flashOn, setFlashOn] = useState(false);
  const [showFallback, setShowFallback] = useState(false);

  const {
    webcamRef,
    facingMode,
    getScreenshot,
    toggleFacingMode,
    handleUserMedia,
    handleUserMediaError,
  } = useCamera();

  // ── Live Scan Hook ──────────────────────────────────────────────────────────
  const {
    isScanning,
    setIsScanning,
    isLiveProcessing,
    liveScanActive,
    startLiveScan,
    stopLiveScan,
    scanError,
  } = useScanPlate({
    onSuccess: (plate) => {
      // Live scan should immediately navigate (no confirmation step)
      navigate(`/mobile/vehicle/${encodeURIComponent(plate.plateNumber)}`);
    },
  });

  // ── Photo Capture Hook ──────────────────────────────────────────────────────
  const {
    state: photoState,
    capturedImageSrc,
    detectedPlate: photoPlate,
    editedPlate,
    isEditing,
    error: photoError,
    captureAndProcess,
    retry: photoRetry,
    toggleEdit,
    updateEditedPlate,
    confirmPlate,
  } = usePhotoCapture({
    onConfirm: (plate) => {
      navigate(`/mobile/vehicle/${encodeURIComponent(plate.plateNumber)}`);
    },
  });

  // 30-second fallback timer for manual entry (live mode only)
  useEffect(() => {
    let timer: NodeJS.Timeout;
    if (liveScanActive) {
      setShowFallback(false);
      timer = setTimeout(() => {
        setShowFallback(true);
      }, 30000);
    } else {
      setShowFallback(false);
    }
    return () => clearTimeout(timer);
  }, [liveScanActive]);

  const handleCapture = useCallback(async () => {
    setIsScanning(true);
    await captureAndProcess(getScreenshot);
    setIsScanning(false);
  }, [captureAndProcess, getScreenshot, setIsScanning]);

  const handlePhotoRetry = useCallback(() => {
    photoRetry();
  }, [photoRetry]);

  const toggleLiveScan = useCallback(() => {
    if (liveScanActive) {
      stopLiveScan();
    } else {
      startLiveScan(getScreenshot);
    }
  }, [liveScanActive, startLiveScan, stopLiveScan, getScreenshot]);

  const handleManualEntry = () => {
    navigate('/mobile/search');
  };

  // Determine which result card to show
  const showPhotoResult = mode === 'photo' && photoState === 'result' && photoPlate;
  const showResultCard = showPhotoResult;

  return (
    <MobileLayout title={t('mobileScan.title')} hideNavigation>
      <div className="relative flex h-[calc(100dvh-4rem)] flex-col bg-black overflow-hidden">
        <ScanViewfinder
          webcamRef={webcamRef}
          facingMode={facingMode}
          isScanning={
            isScanning ||
            photoState === 'processing' ||
            (mode === 'live' && liveScanActive && isLiveProcessing)
          }
          mode={mode}
          liveScanActive={liveScanActive}
          capturedImageSrc={mode === 'photo' ? capturedImageSrc : null}
          onUserMedia={handleUserMedia}
          onUserMediaError={handleUserMediaError}
        />

        <ScanTopControls
          onClose={() => navigate(-1)}
          onToggleFlash={() => setFlashOn(!flashOn)}
          onToggleFacingMode={toggleFacingMode}
          flashOn={flashOn}
          facingMode={facingMode}
        />

        {/* Error banners */}
        {scanError && liveScanActive && (
          <div className="absolute top-24 left-1/2 -translate-x-1/2 z-20 px-4 py-2 bg-destructive/90 text-destructive-foreground rounded-full text-sm font-medium animate-in fade-in slide-in-from-top-4 shadow-lg whitespace-nowrap">
            {scanError}
          </div>
        )}

        {photoError && mode === 'photo' && (
          <div className="absolute top-24 left-1/2 -translate-x-1/2 z-20 px-4 py-2 bg-destructive/90 text-destructive-foreground rounded-full text-sm font-medium animate-in fade-in slide-in-from-top-4 shadow-lg whitespace-nowrap">
            {photoError}
          </div>
        )}

        {/* Photo capture hint */}
        {mode === 'photo' && photoState === 'idle' && (
          <div className="absolute top-20 left-0 right-0 flex justify-center z-10 pointer-events-none">
            <p className="text-white/70 text-xs font-medium bg-black/40 px-3 py-1.5 rounded-full backdrop-blur-sm">
              {t('mobileScan.captureHint')}
            </p>
          </div>
        )}

        {/* Live scan hint */}
        {mode === 'live' && liveScanActive && (
          <div className="absolute top-20 left-0 right-0 flex justify-center z-10 pointer-events-none">
            <p className="text-white/70 text-xs font-medium bg-black/40 px-3 py-1.5 rounded-full backdrop-blur-sm">
              {t('mobileScan.liveScanHint')}
            </p>
          </div>
        )}

        {/* Live mode 30s fallback */}
        {showFallback && (
          <div className="absolute top-1/2 left-0 right-0 z-20 flex flex-col items-center px-6 animate-in fade-in slide-in-from-bottom-4">
            <div className="bg-black/80 backdrop-blur-md p-6 rounded-2xl border border-white/20 text-center shadow-2xl">
              <p className="text-white mb-4 font-medium">{t('mobileScan.takingTooLong')}</p>
              <Button
                onClick={handleManualEntry}
                className="w-full gap-2 bg-accent text-accent-foreground hover:bg-accent/90"
              >
                <Keyboard className="h-4 w-4" />
                {t('mobileScan.manualEntry')}
              </Button>
            </div>
          </div>
        )}

        {/* Result confirmation card — shared by BOTH photo and live modes */}
        {showPhotoResult && (
          <ScanResultCard
            detectedPlate={photoPlate}
            isEditing={isEditing}
            editedPlate={editedPlate}
            onEditToggle={toggleEdit}
            onEditChange={updateEditedPlate}
            onRetry={handlePhotoRetry}
            onConfirm={confirmPlate}
          />
        )}

        {/* Only show action buttons when there's no result card */}
        {!showResultCard && (
          <ScanActionButtons
            mode={mode}
            onModeChange={(newMode) => {
              setMode(newMode);
              stopLiveScan();
              photoRetry(); // Reset photo state when switching modes
            }}
            liveScanActive={liveScanActive}
            onToggleLiveScan={toggleLiveScan}
            onCapture={handleCapture}
            isScanning={isScanning || photoState === 'processing'}
            hasResult={false}
          />
        )}
      </div>
    </MobileLayout>
  );
}
