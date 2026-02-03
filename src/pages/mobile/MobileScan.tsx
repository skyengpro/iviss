import { useState, useRef, useCallback } from "react";
import { MobileLayout } from "@/components/layout/MobileLayout";
import { useSearchParams, useNavigate } from "react-router-dom";
import Webcam from "react-webcam";

import { usePlateScanner, DetectedPlate } from "@/hooks/usePlateScanner";
import { ScanViewfinder } from "@/components/mobile/scan/ScanViewfinder";
import { ScanTopControls } from "@/components/mobile/scan/ScanTopControls";
import { ScanDetectionsList } from "@/components/mobile/scan/ScanDetectionsList";
import { ScanResultCard } from "@/components/mobile/scan/ScanResultCard";
import { ScanActionButtons } from "@/components/mobile/scan/ScanActionButtons";

type ScanMode = "photo" | "live";

export default function MobileScan() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const initialMode = (searchParams.get("mode") as ScanMode) || "photo";

  const [mode, setMode] = useState<ScanMode>(initialMode);
  const [flashOn, setFlashOn] = useState(false);
  const [facingMode, setFacingMode] = useState<"user" | "environment">("environment");

  const [detectedPlate, setDetectedPlate] = useState<DetectedPlate | null>(null);
  const [editedPlate, setEditedPlate] = useState("");
  const [isEditing, setIsEditing] = useState(false);

  const webcamRef = useRef<Webcam>(null);

  const {
    isScanning,
    setIsScanning,
    useDemoData,
    setUseDemoData,
    liveScanActive,
    liveDetections,
    processImage,
    startLiveScan,
    stopLiveScan,
  } = usePlateScanner({
    onCriticalDetection: (plate) => setDetectedPlate(plate),
    initialUseDemoData: true
  });

  const handleCapture = async () => {
    if (!webcamRef.current) return;
    setIsScanning(true);

    const imageSrc = webcamRef.current.getScreenshot();

    if (imageSrc) {
      try {
        const result = await processImage(imageSrc);

        if (result) {
          const detection: DetectedPlate = {
            plateNumber: result.plateNumber,
            confidence: result.confidence,
            status: (result as any).status || 'valid',
          };
          setDetectedPlate(detection);
          setEditedPlate(result.plateNumber);
        } else {
          setDetectedPlate({
            plateNumber: "NO PLATE DETECTED",
            confidence: 0,
            status: 'warning'
          });
          setEditedPlate("");
        }
      } catch (e) {
        setDetectedPlate({
          plateNumber: "OCR ERROR",
          confidence: 0,
          status: 'warning'
        });
        setEditedPlate("");
      }
    }

    setIsScanning(false);
  };

  const toggleLiveScan = useCallback(() => {
    if (liveScanActive) {
      stopLiveScan();
    } else {
      startLiveScan(webcamRef.current);
    }
  }, [liveScanActive, startLiveScan, stopLiveScan]);

  const handleConfirm = () => {
    const plateToSearch = isEditing ? editedPlate : detectedPlate?.plateNumber;
    if (plateToSearch) {
      navigate(`/mobile/vehicle/${encodeURIComponent(plateToSearch)}`);
    }
  };

  const handleRetry = () => {
    setDetectedPlate(null);
    setEditedPlate("");
    setIsEditing(false);
  };

  const handleLivePlateClick = (plate: DetectedPlate) => {
    setDetectedPlate(plate);
    setEditedPlate(plate.plateNumber);
    stopLiveScan();
  };

  return (
    <MobileLayout title="Scan Plate" hideNavigation>
      <div className="relative flex h-[calc(100dvh-4rem)] flex-col bg-black overflow-hidden">

        <ScanViewfinder
          webcamRef={webcamRef}
          facingMode={facingMode}
          isScanning={isScanning}
          mode={mode}
          liveScanActive={liveScanActive}
        />

        <ScanTopControls
          onClose={() => navigate(-1)}
          onToggleFlash={() => setFlashOn(!flashOn)}
          onToggleFacingMode={() => setFacingMode(prev => prev === "user" ? "environment" : "user")}
          flashOn={flashOn}
          facingMode={facingMode}
          useDemoData={useDemoData}
          onToggleDemoData={setUseDemoData}
        />

        <ScanDetectionsList
          detections={liveDetections}
          onPlateClick={handleLivePlateClick}
        />

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
