import React from "react";
import { Camera, Radio } from "lucide-react";
import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";

interface ScanActionButtonsProps {
  mode: 'photo' | 'live';
  onModeChange: (mode: 'photo' | 'live') => void;
  liveScanActive: boolean;
  onToggleLiveScan: () => void;
  onCapture: () => void;
  isScanning: boolean;
  hasResult: boolean;
}

export const ScanActionButtons: React.FC<ScanActionButtonsProps> = ({
  mode,
  onModeChange,
  liveScanActive,
  onToggleLiveScan,
  onCapture,
  isScanning,
  hasResult,
}) => {
    const { t } = useTranslation();
    if (hasResult) return null;

    return (
        <>
            {/* Mode toggle */}
            <div className="absolute bottom-24 left-0 right-0 flex justify-center z-10">
                <div className="flex rounded-full bg-black/60 p-1">
                    <button
                        onClick={() => onModeChange("photo")}
                        className={cn(
                            "flex items-center gap-2 rounded-full px-4 py-2 text-sm font-medium transition-colors",
                            mode === "photo"
                                ? "bg-accent text-accent-foreground"
                                : "text-white/70 hover:text-white"
                        )}
                    >
                        <Camera className="h-4 w-4" />
                        {t('mobileScan.photo')}
                    </button>
                    <button
                        onClick={() => onModeChange("live")}
                        className={cn(
                            "flex items-center gap-2 rounded-full px-4 py-2 text-sm font-medium transition-colors",
                            mode === "live"
                                ? "bg-accent text-accent-foreground"
                                : "text-white/70 hover:text-white"
                        )}
                    >
                        <Radio className="h-4 w-4" />
                        {t('mobileScan.live')}
                    </button>
                </div>
            </div>

            {/* Main Action Button */}
            <div className="absolute bottom-8 left-0 right-0 flex justify-center z-10">
                {mode === "photo" ? (
                    !isScanning && (
                        <button
                            onClick={onCapture}
                            className="flex h-20 w-20 items-center justify-center rounded-full border-4 border-white bg-white/20 transition-transform active:scale-95"
                        >
                            <div className="h-14 w-14 rounded-full bg-white" />
                        </button>
                    )
                ) : (
                    <button
                        onClick={onToggleLiveScan}
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
                                <span className="text-sm font-medium">{t('mobileScan.stopScanning')}</span>
                            </>
                        ) : (
                            <>
                                <Radio className="h-4 w-4" />
                                <span className="text-sm font-medium">{t('mobileScan.startLiveScan')}</span>
                            </>
                        )}
                    </button>
                )}
            </div>
        </>
    );
};
