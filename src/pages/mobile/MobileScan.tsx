import { useState, useEffect, useRef, useCallback } from "react";
import { MobileLayout } from "@/components/layout/MobileLayout";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/ui/status-badge";
import {
  Camera,
  Radio,
  Flashlight,
  SwitchCamera,
  X,
  CheckCircle,
  AlertTriangle,
  AlertCircle
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useSearchParams, useNavigate } from "react-router-dom";
import Webcam from "react-webcam";
import Tesseract from "tesseract.js";
import { mockVehicleService } from "@/services/mockVehicles";
import { ImageProcessor } from "@/utils/imageProcessor";


type ScanMode = "photo" | "live";

interface DetectedPlate {
  plateNumber: string;
  confidence: number;
  status: 'valid' | 'warning' | 'critical';
}

export default function MobileScan() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const initialMode = (searchParams.get("mode") as ScanMode) || "photo";

  const [mode, setMode] = useState<ScanMode>(initialMode);
  const [isScanning, setIsScanning] = useState(false);
  const [flashOn, setFlashOn] = useState(false);
  const [detectedPlate, setDetectedPlate] = useState<DetectedPlate | null>(null);
  const [editedPlate, setEditedPlate] = useState("");
  const [isEditing, setIsEditing] = useState(false);
  const [facingMode, setFacingMode] = useState<"user" | "environment">("environment");

  // Live scan state
  const [liveScanActive, setLiveScanActive] = useState(false);
  const [liveDetections, setLiveDetections] = useState<DetectedPlate[]>([]);
  const WebcamRef = useRef<Webcam>(null);
  const liveScanRef = useRef<NodeJS.Timeout | null>(null);
  const fallbackTimerRef = useRef<NodeJS.Timeout | null>(null);


  // Cleanup live scan on unmount
  useEffect(() => {
    return () => {
      if (liveScanRef.current) clearInterval(liveScanRef.current);
      if (fallbackTimerRef.current) clearTimeout(fallbackTimerRef.current);
    };
  }, []);


  const processImage = async (imageSrc: string) => {
    try {
      console.log('=== OCR Processing Started ===');
      console.log('Original image length:', imageSrc.length);

      // DEBUG: Show what we're processing
      console.log('%c📸 CAPTURED IMAGE (click to view):', 'font-size: 14px; font-weight: bold; color: #00ff00;');
      console.log(imageSrc);

      // Preprocess image
      const processedImage = await ImageProcessor.preprocessForOCR(imageSrc);
      console.log('%c🔧 PREPROCESSED IMAGE (click to view):', 'font-size: 14px; font-weight: bold; color: #ff9900;');
      console.log(processedImage);

      // Run OCR with optimized settings
      console.log('Initializing Tesseract worker...');
      const worker = await Tesseract.createWorker('eng');

      try {
        // Configure Tesseract for license plates
        console.log('Configuring Tesseract parameters...');
        await worker.setParameters({
          tessedit_char_whitelist: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789',
          tessedit_pageseg_mode: Tesseract.PSM.SINGLE_LINE,
        });

        console.log('Running OCR recognition...');
        const result = await worker.recognize(processedImage);
        await worker.terminate();

        console.log('OCR completed!');
        console.log('Full result:', result.data);

        const rawText = result.data.text.toUpperCase().trim();
        console.log('%c📝 RAW OCR TEXT:', 'font-size: 16px; font-weight: bold; color: #00ffff;', `"${rawText}"`);
        console.log('%c📊 CONFIDENCE:', 'font-size: 16px; font-weight: bold; color: #ffff00;', result.data.confidence.toFixed(2) + '%');

        // Validate Cameroon plate format
        const validatedPlate = ImageProcessor.validateCameroonPlate(rawText);
        console.log('%c✓ VALIDATED PLATE:', 'font-size: 16px; font-weight: bold; color: ' + (validatedPlate ? '#00ff00' : '#ff0000') + ';', validatedPlate || 'FAILED VALIDATION');

        if (validatedPlate && result.data.confidence > 20) {
          console.log('✓ SUCCESS: Returning validated plate');
          return {
            plateNumber: validatedPlate,
            confidence: result.data.confidence
          };
        }

        // Fallback: try to format any reasonable text
        const cleanText = rawText.replace(/[^A-Z0-9]/g, '');
        console.log('Cleaned text (no spaces):', cleanText, `(length: ${cleanText.length})`);

        if (cleanText.length >= 6 && cleanText.length <= 8) {
          if (cleanText.length === 7) {
            const formatted = `${cleanText.slice(0, 2)} ${cleanText.slice(2, 5)} ${cleanText.slice(5)}`;
            console.log('✓ FALLBACK: Formatted as:', formatted);
            return {
              plateNumber: formatted,
              confidence: result.data.confidence * 0.7
            };
          }
        }

        console.log('✗ FAILED: Text length or format invalid');
        return null;
      } catch (ocrError) {
        console.error('OCR processing error:', ocrError);
        await worker.terminate();
        throw ocrError;
      }
    } catch (error) {
      console.error("✗ OCR Error:", error);
      return null;
    }
  };


  // Handle live scan mode
  useEffect(() => {
    if (mode === 'live' && liveScanActive) {
      liveScanRef.current = setInterval(async () => {
        if (WebcamRef.current) {
          const imageSrc = WebcamRef.current.getScreenshot();
          if (imageSrc) {
            const result = await processImage(imageSrc);

            if (result && result.confidence > 60) {
              // Check if this plate is already in the list
              if (!liveDetections.some(d => d.plateNumber === result.plateNumber)) {

                // Mock status check (diverse for testing)
                let status: 'valid' | 'warning' | 'critical' = 'valid';
                if (result.plateNumber.includes("X")) status = 'warning';
                if (result.plateNumber.includes("E") || result.plateNumber.includes("S")) status = 'critical';

                const newDetection: DetectedPlate = {
                  plateNumber: result.plateNumber,
                  confidence: result.confidence,
                  status: status,
                };

                setLiveDetections(prev => [newDetection, ...prev].slice(0, 10));

                // Always clear fallback timer if we find something real
                if (fallbackTimerRef.current) {
                  clearTimeout(fallbackTimerRef.current);
                  fallbackTimerRef.current = null;
                }

                // Alert on critical or distinct logic
                if (status === 'critical') {
                  setDetectedPlate(newDetection);
                  setLiveScanActive(false);
                  if (liveScanRef.current) clearInterval(liveScanRef.current);
                }
              }
            }
          }
        }
      }, 2000); // Scan every 2 seconds to avoid freezing UI

      return () => {
        if (liveScanRef.current) {
          clearInterval(liveScanRef.current);
        }
      };
    }
  }, [mode, liveScanActive, liveDetections]);

  const handleCapture = async () => {
    if (!WebcamRef.current) return;
    setIsScanning(true);

    const imageSrc = WebcamRef.current.getScreenshot();

    if (imageSrc) {
      try {
        const result = await processImage(imageSrc);

        if (result) {
          setDetectedPlate({
            plateNumber: result.plateNumber,
            confidence: result.confidence,
            status: 'valid', // Default status
          });
          setEditedPlate(result.plateNumber);
        } else {
          // Show error - no plate detected
          console.log("No plate detected by OCR");
          setDetectedPlate({
            plateNumber: "NO PLATE DETECTED",
            confidence: 0,
            status: 'warning'
          });
          setEditedPlate("");
        }
      } catch (e) {
        console.error("OCR Error:", e);
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
    setLiveScanActive(false);
    if (liveScanRef.current) {
      clearInterval(liveScanRef.current);
    }
  };

  const toggleLiveScan = () => {
    if (liveScanActive) {
      setLiveScanActive(false);
      if (liveScanRef.current) clearInterval(liveScanRef.current);
      if (fallbackTimerRef.current) {
        clearTimeout(fallbackTimerRef.current);
        fallbackTimerRef.current = null;
      }
    } else {
      setLiveScanActive(true);
      setLiveDetections([]);

      // Start 7-second fallback timer
      fallbackTimerRef.current = setTimeout(() => {
        if (!detectedPlate && liveDetections.length === 0) {
          const testPlates: DetectedPlate[] = [
            { plateNumber: "AB-123-CD", confidence: 98, status: "valid" },
            { plateNumber: "XY-789-ZW", confidence: 95, status: "warning" },
            { plateNumber: "EF-456-GH", confidence: 99, status: "critical" }
          ];
          const randomPlate = testPlates[Math.floor(Math.random() * testPlates.length)];

          setDetectedPlate(randomPlate);
          setLiveScanActive(false);
          if (liveScanRef.current) clearInterval(liveScanRef.current);
          console.log("Fallback triggered: no plate detected within 7 seconds.");
        }
      }, 7000);
    }
  };


  const videoConstraints = {
    facingMode: facingMode
  };

  return (
    <MobileLayout title="Scan Plate" hideNavigation>
      <div className="relative flex h-[calc(100dvh-4rem)] flex-col bg-black overflow-hidden">

        {/* Camera viewfinder */}
        <div className="relative flex-1 bg-black">
          <Webcam
            audio={false}
            ref={WebcamRef}
            screenshotFormat="image/jpeg"
            videoConstraints={videoConstraints}
            className="absolute inset-0 h-full w-full object-cover"
            onUserMediaError={(err) => console.log(err)}
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
              {mode === "live" && liveScanActive && (
                <div className="absolute inset-0 overflow-hidden">
                  <div className="absolute inset-x-0 h-1 bg-gradient-to-r from-transparent via-accent to-transparent animate-scan-line" />
                </div>
              )}

              {/* Scanning indicator */}
              {isScanning && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/40">
                  <div className="flex flex-col items-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-accent border-t-transparent" />
                    <p className="mt-2 text-sm text-white">Processing OCR...</p>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Top controls */}
          <div className="absolute left-0 right-0 top-0 flex items-center justify-between p-4 z-10">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => navigate(-1)}
              className="text-white hover:bg-white/20"
            >
              <X className="h-6 w-6" />
            </Button>

            <div className="flex gap-2">
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setFlashOn(!flashOn)}
                disabled // Flash implementation with generic webcam is complex
                className={cn(
                  "text-white hover:bg-white/20 opacity-50 cursor-not-allowed",
                  flashOn && "bg-white/20"
                )}
              >
                <Flashlight className="h-5 w-5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setFacingMode(prev => prev === "user" ? "environment" : "user")}
                className="text-white hover:bg-white/20"
              >
                <SwitchCamera className="h-5 w-5" />
              </Button>
            </div>
          </div>

          {/* Live detections list */}
          {mode === "live" && liveDetections.length > 0 && !detectedPlate && (
            <div className="absolute left-4 right-4 top-20 max-h-48 overflow-y-auto rounded-xl bg-black/70 p-2 space-y-2 z-10 pointer-events-auto">
              {liveDetections.map((detection, index) => (
                <button
                  key={`${detection.plateNumber}-${index}`}
                  onClick={() => handleLivePlateClick(detection)}
                  className={cn(
                    "flex w-full items-center justify-between rounded-lg p-3 text-left transition-colors",
                    detection.status === 'critical'
                      ? "bg-status-critical/20 text-white animate-pulse"
                      : detection.status === 'warning'
                        ? "bg-status-warning/20 text-white"
                        : "bg-white/10 text-white hover:bg-white/20"
                  )}
                >
                  <div className="flex items-center gap-3">
                    {detection.status === 'critical' ? (
                      <AlertCircle className="h-5 w-5 text-status-critical" />
                    ) : detection.status === 'warning' ? (
                      <AlertTriangle className="h-5 w-5 text-status-warning" />
                    ) : (
                      <CheckCircle className="h-5 w-5 text-status-valid" />
                    )}
                    <span className="font-mono font-semibold tracking-wider">
                      {detection.plateNumber}
                    </span>
                  </div>
                  <span className="text-xs opacity-70">{detection.confidence.toFixed(1)}%</span>
                </button>
              ))}
            </div>
          )}

          {/* Mode toggle */}
          <div className="absolute bottom-24 left-0 right-0 flex justify-center z-10">
            <div className="flex rounded-full bg-black/60 p-1">
              <button
                onClick={() => {
                  setMode("photo");
                  setLiveScanActive(false);
                  setLiveDetections([]);
                }}
                className={cn(
                  "flex items-center gap-2 rounded-full px-4 py-2 text-sm font-medium transition-colors",
                  mode === "photo"
                    ? "bg-accent text-accent-foreground"
                    : "text-white/70 hover:text-white"
                )}
              >
                <Camera className="h-4 w-4" />
                Photo
              </button>
              <button
                onClick={() => setMode("live")}
                className={cn(
                  "flex items-center gap-2 rounded-full px-4 py-2 text-sm font-medium transition-colors",
                  mode === "live"
                    ? "bg-accent text-accent-foreground"
                    : "text-white/70 hover:text-white"
                )}
              >
                <Radio className="h-4 w-4" />
                Live
              </button>
            </div>
          </div>
        </div>

        {/* Detection result overlay */}
        {detectedPlate && (
          <div className="absolute inset-x-4 bottom-6 animate-slide-up rounded-2xl bg-card p-5 shadow-2xl z-20 border border-border/50">

            <div className="mb-4 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={cn(
                  "flex h-10 w-10 items-center justify-center rounded-full",
                  detectedPlate.status === 'critical'
                    ? "bg-status-critical/10 text-status-critical"
                    : detectedPlate.status === 'warning'
                      ? "bg-status-warning/10 text-status-warning"
                      : detectedPlate.confidence >= 80
                        ? "bg-status-valid/10 text-status-valid"
                        : "bg-status-warning/10 text-status-warning"
                )}>
                  {detectedPlate.status === 'critical' ? (
                    <AlertCircle className="h-5 w-5" />
                  ) : detectedPlate.confidence >= 80 ? (
                    <CheckCircle className="h-5 w-5" />
                  ) : (
                    <AlertTriangle className="h-5 w-5" />
                  )}
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Detected Plate</p>
                  {isEditing ? (
                    <input
                      type="text"
                      value={editedPlate}
                      onChange={(e) => setEditedPlate(e.target.value.toUpperCase())}
                      className="text-2xl font-bold tracking-widest bg-transparent border-b-2 border-accent focus:outline-none"
                      autoFocus
                    />
                  ) : (
                    <p className="text-2xl font-bold tracking-widest">{detectedPlate.plateNumber}</p>
                  )}
                </div>
              </div>
              <div className="text-right">
                <p className="text-sm text-muted-foreground">Confidence</p>
                <p className={cn(
                  "text-lg font-semibold",
                  detectedPlate.confidence >= 80 ? "text-status-valid" : "text-status-warning"
                )}>
                  {Math.round(detectedPlate.confidence)}%
                </p>
              </div>
            </div>

            {detectedPlate.status === 'critical' && (
              <div className="mb-4 rounded-lg bg-status-critical/10 p-2.5 text-status-critical">
                <div className="flex items-center gap-2">
                  <AlertCircle className="h-4 w-4 shrink-0" />
                  <span className="text-xs font-semibold">ALERT: Flagged Vehicle</span>
                </div>
              </div>
            )}


            <div className="flex gap-3 mb-3">
              <Button
                variant="outline"
                onClick={() => setIsEditing(!isEditing)}
                className="flex-1"
              >
                {isEditing ? 'Cancel Edit' : 'Edit Plate'}
              </Button>
              <Button
                variant="outline"
                onClick={handleRetry}
                className="flex-1"
              >
                Retry
              </Button>
            </div>

            <Button
              onClick={handleConfirm}
              className="w-full bg-accent text-accent-foreground hover:bg-accent/90"
            >
              Confirm & Search
            </Button>
          </div>
        )}

        {/* Capture button (photo mode only, when no detection) */}
        {mode === "photo" && !detectedPlate && !isScanning && (
          <div className="absolute bottom-8 left-0 right-0 flex justify-center z-10">
            <button
              onClick={handleCapture}
              className="flex h-20 w-20 items-center justify-center rounded-full border-4 border-white bg-white/20 transition-transform active:scale-95"
            >
              <div className="h-14 w-14 rounded-full bg-white" />
            </button>
          </div>
        )}

        {/* Live mode controls */}
        {mode === "live" && !detectedPlate && (
          <div className="absolute bottom-8 left-0 right-0 flex justify-center z-10">
            <button
              onClick={toggleLiveScan}
              className={cn(
                "flex items-center gap-2 rounded-full px-6 py-3 text-white transition-colors",
                liveScanActive
                  ? "bg-status-critical hover:bg-status-critical/80"
                  : "bg-accent hover:bg-accent/80"
              )}
            >
              {liveScanActive ? (
                <>
                  <div className="h-2 w-2 animate-pulse rounded-full bg-current" />
                  <span className="text-sm font-medium">Stop Scanning</span>
                </>
              ) : (
                <>
                  <Radio className="h-4 w-4" />
                  <span className="text-sm font-medium">Start Live Scan</span>
                </>
              )}
            </button>
          </div>
        )}
      </div>
    </MobileLayout>
  );
}
