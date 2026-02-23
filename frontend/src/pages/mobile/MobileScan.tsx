import { useState, useCallback, useEffect } from 'react';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useScanPlate, DetectedPlate } from '@/hooks/feature/useScanPlate';
import { useCamera } from '@/hooks/feature/useCamera';
import { ScanViewfinder } from '@/components/mobile/scan/ScanViewfinder';
import { ScanTopControls } from '@/components/mobile/scan/ScanTopControls';
import { ScanDetectionsList } from '@/components/mobile/scan/ScanDetectionsList';
import { ScanResultCard } from '@/components/mobile/scan/ScanResultCard';
import { ScanActionButtons } from '@/components/mobile/scan/ScanActionButtons';
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

  const [detectedPlate, setDetectedPlate] = useState<DetectedPlate | null>(null);
  const [editedPlate, setEditedPlate] = useState('');
  const [isEditing, setIsEditing] = useState(false);

  const {
    webcamRef,
    facingMode,
    setFacingMode,
    getScreenshot,
    toggleFacingMode,
    handleUserMedia,
    handleUserMediaError,
  } = useCamera();

  const {
    isScanning,
    setIsScanning,
    useDemoData,
    setUseDemoData,
    liveScanActive,
    liveDetections,
    startLiveScan,
    stopLiveScan,
    scanError,
  } = useScanPlate({
    onSuccess: (plate) => {
      setDetectedPlate(plate);
      setEditedPlate(plate.plateNumber);
    },
    initialUseDemoData: true,
  });

  // 30-second fallback timer for manual entry
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

  const handleCapture = async () => {
    setIsScanning(true);
    const imageSrc = getScreenshot();

    if (imageSrc) {
      try {
        // Manual capture uses the same processing logic but independently of the stability loop
        // For simplicity, we could just reuse the processFrame from useScanPlate if exported
        // or just let the user know we found nothing if it's not a live scan.
        // In this architecture, we focus on live scan stability.
      } catch (e) {
        console.error('Capture failed:', e);
      }
    }
    setIsScanning(false);
  };

  const toggleLiveScan = useCallback(() => {
    if (liveScanActive) {
      stopLiveScan();
    } else {
      startLiveScan(getScreenshot);
    }
  }, [liveScanActive, startLiveScan, stopLiveScan, getScreenshot]);

  const handleConfirm = () => {
    const plateToSearch = isEditing ? editedPlate : detectedPlate?.plateNumber;
    if (plateToSearch) {
      navigate(`/mobile/vehicle/${encodeURIComponent(plateToSearch)}`);
    }
  };

  const handleRetry = () => {
    setDetectedPlate(null);
    setEditedPlate('');
    setIsEditing(false);
    stopLiveScan(); // Ensure hook state is also reset
  };

  const handleLivePlateClick = (plate: DetectedPlate) => {
    setDetectedPlate(plate);
    setEditedPlate(plate.plateNumber);
    stopLiveScan();
  };

  const handleManualEntry = () => {
    navigate('/mobile/search');
  };

  return (
    <MobileLayout title={t('mobileScan.title')} hideNavigation>
      <div className="relative flex h-[calc(100dvh-4rem)] flex-col bg-black overflow-hidden">
        <ScanViewfinder
          webcamRef={webcamRef}
          facingMode={facingMode}
          isScanning={isScanning}
          mode={mode}
          liveScanActive={liveScanActive}
          onUserMedia={handleUserMedia}
          onUserMediaError={handleUserMediaError}
        />

        <ScanTopControls
          onClose={() => navigate(-1)}
          onToggleFlash={() => setFlashOn(!flashOn)}
          onToggleFacingMode={toggleFacingMode}
          flashOn={flashOn}
          facingMode={facingMode}
          useDemoData={useDemoData}
          onToggleDemoData={setUseDemoData}
        />

        {scanError && liveScanActive && !detectedPlate && (
          <div className="absolute top-24 left-1/2 -translate-x-1/2 z-20 px-4 py-2 bg-destructive/90 text-destructive-foreground rounded-full text-sm font-medium animate-in fade-in slide-in-from-top-4 shadow-lg whitespace-nowrap">
            {scanError}
          </div>
        )}

        <ScanDetectionsList detections={liveDetections} onPlateClick={handleLivePlateClick} />

        {showFallback && !detectedPlate && (
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

        {detectedPlate && (
          <ScanResultCard
            detectedPlate={detectedPlate}
            isEditing={isEditing}
            editedPlate={editedPlate}
            onEditToggle={() => setIsEditing(!isEditing)}
            onEditChange={setEditedPlate}
            onRetry={handleRetry}
            onConfirm={handleConfirm}
          />
        )}

        <ScanActionButtons
          mode={mode}
          onModeChange={(newMode) => {
            setMode(newMode);
            stopLiveScan();
          }}
          liveScanActive={liveScanActive}
          onToggleLiveScan={toggleLiveScan}
          onCapture={handleCapture}
          isScanning={isScanning}
          hasResult={!!detectedPlate}
        />
      </div>
    </MobileLayout>
  );
}
