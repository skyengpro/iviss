import { useTranslation } from 'react-i18next';
import { useState, useRef } from 'react';
import { X, Keyboard, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { ImageProcessor } from '@/utils/imageProcessor';

interface PlateInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  isLoading?: boolean;
  className?: string;
  placeholder?: string;
}

export const isValidPlate = (plate: string): boolean => {
  return ImageProcessor.classifyCameroonPlate(plate) !== null;
};

export function PlateInput({
  value,
  onChange,
  onSubmit,
  isLoading,
  className,
  placeholder = 'CE 123 BC',
}: Readonly<PlateInputProps>) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const [isFocused, setIsFocused] = useState(false);
  const [showValidationError, setShowValidationError] = useState(false);

  // Flexible formatting: uppercase and allow valid characters
  const formatPlate = (input: string): string => {
    return input
      .toUpperCase()
      .replace(/\s+/g, ' ') // Collapse multiple spaces
      .replace(/[^A-Z0-9 ]/g, '') // Remove invalid chars
      .trimStart(); // Allow spaces only between parts
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const formatted = formatPlate(e.target.value);
    onChange(formatted);

    // Hide validation error while typing
    if (showValidationError) {
      setShowValidationError(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSubmit();
    }
  };

  const handleSubmit = () => {
    if (!isValidPlate(value)) {
      setShowValidationError(true);
      return;
    }

    setShowValidationError(false);
    onSubmit();
  };

  const handleClear = () => {
    onChange('');
    setShowValidationError(false);
    inputRef.current?.focus();
  };

  const isValid = isValidPlate(value);

  return (
    <div className={cn('relative', className)}>
      <div
        className={cn(
          'flex items-center gap-2 rounded-xl border-2 bg-card p-2 transition-all duration-200',
          showValidationError
            ? 'border-destructive ring-4 ring-destructive/20'
            : isFocused
              ? 'border-accent ring-4 ring-accent/20'
              : 'border-border hover:border-muted-foreground/30'
        )}
      >
        {/* Plate icon/prefix */}
        <div
          className={cn(
            'flex h-12 w-12 shrink-0 items-center justify-center rounded-lg transition-colors',
            showValidationError
              ? 'bg-destructive text-destructive-foreground'
              : 'bg-primary text-primary-foreground'
          )}
        >
          <Keyboard className="h-5 w-5" />
        </div>

        {/* Input */}
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
          placeholder={placeholder}
          className="flex-1 bg-transparent text-xl font-bold tracking-widest placeholder:text-muted-foreground/50 placeholder:font-normal placeholder:tracking-normal focus:outline-none"
          autoComplete="off"
          autoCapitalize="characters"
          maxLength={16}
        />

        {/* Clear button */}
        {value && (
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={handleClear}
            className="shrink-0 text-muted-foreground hover:text-foreground"
          >
            <X className="h-5 w-5" />
          </Button>
        )}
      </div>

      {/* Validation messages */}
      {showValidationError && (
        <div className="mt-2 flex items-center justify-center gap-2 text-sm text-destructive">
          <AlertCircle className="h-4 w-4" />
          <p>{t('mobileSearch.invalidFormat', 'Invalid format. Please check the plate number.')}</p>
        </div>
      )}

      {value.length > 0 && !isValid && !showValidationError && (
        <p className="mt-2 text-center text-sm text-muted-foreground">
          {t('mobileSearch.formatExample', 'Enter a valid Cameroon plate format')}
        </p>
      )}
    </div>
  );
}
