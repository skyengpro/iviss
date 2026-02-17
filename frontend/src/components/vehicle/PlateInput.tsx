import { useTranslation } from 'react-i18next';
import { useState, useRef } from 'react';
import { Search, X, Keyboard, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface PlateInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  isLoading?: boolean;
  className?: string;
  placeholder?: string;
}

// Strict license plate format: 2 letters + 3 numbers + 2 letters (e.g., "WE 234 SD")
const PLATE_REGEX = /^[A-Z]{2}\s\d{3}\s[A-Z]{2}$/;

export function PlateInput({
  value,
  onChange,
  onSubmit,
  isLoading,
  className,
  placeholder = 'WE 234 SD',
}: Readonly<PlateInputProps>) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const [isFocused, setIsFocused] = useState(false);
  const [showValidationError, setShowValidationError] = useState(false);

  // Auto-format plate number with proper spacing: XX 123 YY
  const formatPlate = (input: string): string => {
    // Remove all non-alphanumeric characters
    const cleaned = input.toUpperCase().replace(/[^A-Z0-9]/g, '');

    // Build formatted string: 2 letters + space + 3 numbers + space + 2 letters
    let formatted = '';

    // First 2 letters
    if (cleaned.length > 0) {
      formatted += cleaned.slice(0, 2).replace(/[^A-Z]/g, '');
    }

    // Add space after 2 letters
    if (cleaned.length > 2) {
      formatted += ' ';
      // Next 3 numbers
      formatted += cleaned.slice(2, 5).replace(/[^0-9]/g, '');
    }

    // Add space after numbers
    if (cleaned.length > 5) {
      formatted += ' ';
      // Last 2 letters
      formatted += cleaned.slice(5, 7).replace(/[^A-Z]/g, '');
    }

    return formatted;
  };

  // Validate if the plate matches the required format
  const isValidFormat = (plate: string): boolean => {
    return PLATE_REGEX.test(plate);
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
    if (!isValidFormat(value)) {
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

  const isComplete = value.length === 9; // "XX 123 YY" = 9 characters
  const isValid = isValidFormat(value);

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
        <div className={cn(
          "flex h-12 w-12 shrink-0 items-center justify-center rounded-lg transition-colors",
          showValidationError
            ? "bg-destructive text-destructive-foreground"
            : "bg-primary text-primary-foreground"
        )}>
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
          maxLength={9}
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

        {/* Search button */}
        <Button
          type="button"
          onClick={handleSubmit}
          disabled={!isComplete || !isValid || isLoading}
          className="h-12 w-12 shrink-0 rounded-lg bg-accent text-accent-foreground hover:bg-accent/90 disabled:opacity-50"
        >
          {isLoading ? (
            <div className="h-5 w-5 animate-spin rounded-full border-2 border-current border-t-transparent" />
          ) : (
            <Search className="h-5 w-5" />
          )}
        </Button>
      </div>

      {/* Validation messages */}
      {showValidationError && (
        <div className="mt-2 flex items-center justify-center gap-2 text-sm text-destructive">
          <AlertCircle className="h-4 w-4" />
          <p>
            {t('mobileSearch.invalidFormat', 'Invalid format. Use: WE 234 SD (2 letters, 3 numbers, 2 letters)')}
          </p>
        </div>
      )}

      {value.length > 0 && !isComplete && !showValidationError && (
        <p className="mt-2 text-center text-sm text-muted-foreground">
          {t('mobileSearch.formatExample', 'Format: WE 234 SD (2 letters, 3 numbers, 2 letters)')}
        </p>
      )}
    </div>
  );
}
