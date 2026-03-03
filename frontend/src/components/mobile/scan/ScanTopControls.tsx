import { Flashlight, SwitchCamera, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface ScanTopControlsProps {
  onClose: () => void;
  onToggleFlash: () => void;
  onToggleFacingMode: () => void;
  flashOn: boolean;
  facingMode: 'user' | 'environment';
}

export const ScanTopControls: React.FC<ScanTopControlsProps> = ({
  onClose,
  onToggleFlash,
  onToggleFacingMode,
  flashOn,
  facingMode,
}) => {
  return (
    <div className="absolute left-0 right-0 top-0 flex items-center justify-between p-4 z-20">
      <Button
        variant="ghost"
        size="icon"
        onClick={onClose}
        className="text-white hover:bg-white/20"
      >
        <X className="h-6 w-6" />
      </Button>

      <div className="flex items-center gap-4">
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={onToggleFlash}
            disabled // Flash implementation with generic webcam is complex
            className={cn(
              'text-white hover:bg-white/20 opacity-50 cursor-not-allowed',
              flashOn && 'bg-white/20'
            )}
          >
            <Flashlight className="h-5 w-5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={onToggleFacingMode}
            className="text-white hover:bg-white/20"
          >
            <SwitchCamera className="h-5 w-5" />
          </Button>
        </div>
      </div>
    </div>
  );
};