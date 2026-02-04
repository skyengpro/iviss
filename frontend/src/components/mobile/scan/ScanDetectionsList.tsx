import React from 'react';
import { cn } from '@/lib/utils';
import { CheckCircle, AlertTriangle, AlertCircle } from 'lucide-react';
import { DetectedPlate } from '@/hooks/usePlateScanner';

interface ScanDetectionsListProps {
  detections: DetectedPlate[];
  onPlateClick: (plate: DetectedPlate) => void;
}

export const ScanDetectionsList: React.FC<ScanDetectionsListProps> = ({
  detections,
  onPlateClick,
}) => {
  if (detections.length === 0) return null;

  return (
    <div className="absolute left-4 right-4 top-20 max-h-48 overflow-y-auto rounded-xl bg-black/70 p-2 space-y-2 z-10 pointer-events-auto">
      {detections.map((detection, index) => (
        <button
          key={`${detection.plateNumber}-${index}`}
          onClick={() => onPlateClick(detection)}
          className={cn(
            'flex w-full items-center justify-between rounded-lg p-3 text-left transition-colors',
            detection.status === 'critical'
              ? 'bg-status-critical/20 text-white animate-pulse'
              : detection.status === 'warning'
                ? 'bg-status-warning/20 text-white'
                : 'bg-white/10 text-white hover:bg-white/20'
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
            <span className="font-mono font-semibold tracking-wider">{detection.plateNumber}</span>
          </div>
          <span className="text-xs opacity-70">{detection.confidence.toFixed(1)}%</span>
        </button>
      ))}
    </div>
  );
};
