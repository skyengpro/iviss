import { useTranslation } from 'react-i18next';
import { useState, useRef } from 'react';
import { X, Keyboard, AlertCircle } from 'lucide-react';
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

// 1. Standard: (REGION) 1234 A OR (REGION) 123 AB
// 2. Police: SN 1234
// 3. Military: 1234567
// 4. Government: EN1234X
// 5. Postal: RT123456
// 6. Diplomatic: CD 01 123
const REGION = 'AD|CE|ES|EN|LT|NO|NW|OU|SU|SW';
const PLATE_REGEX = new RegExp(
  `^(?:(?:${REGION})\\s\\d{4}\\s[A-Z]|(?:${REGION})\\s\\d{3}\\s[A-Z]{2}|SN\\s\\d{4}|\\d{7}|[A-Z]{2}\\d{4}[A-Z]|RT\\d{6}|CD\\s\\d{1,3}\\s\\d{1,3})$`
);

export function PlateInput({
  value,
  onChange,
  onSubmit,
  isLoading,
  className,
  placeholder = 'CE 1234 A',
}: Readonly<PlateInputProps>) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const [isFocused, setIsFocused] = useState(false);
  const [showValidationError, setShowValidationError] = useState(false);

  // Auto-format plate number with proper spacing: XX 123 YY
  const formatPlate = (input: string): string => {
    // Standardize to uppercase
    const upper = input.toUpperCase();

    // Remove multiple spaces and limit to 12 chars (safety)
    return upper.replace(/\s+/g, ' ').slice(0, 12);
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

  const isComplete = value.length >= 7; // Shortest valid is 7 chars (Military/Postal)

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
          maxLength={12}
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
        <div className="mt-2 flex flex-col items-center justify-center gap-1 text-sm text-destructive">
          <div className="flex items-center gap-2">
            <AlertCircle className="h-4 w-4" />
            <p>{t('mobileSearch.invalidFormat', 'Invalid plate format')}</p>
          </div>
          <p className="text-xs opacity-80">
            {t('mobileSearch.supportedFormats', 'Example: CE 1234 A, SN 1234, 1234567, RT123456')}
          </p>
        </div>
      )}

      {value.length > 0 && !isComplete && !showValidationError && (
        <p className="mt-2 text-center text-sm text-muted-foreground">
          {t('mobileSearch.formatExample', 'Enter a valid Cameroon license plate')}
        </p>
      )}
    </div>
  );
}
