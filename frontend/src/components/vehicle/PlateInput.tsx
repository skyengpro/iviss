import { useTranslation } from 'react-i18next';
import { useState, useRef } from 'react';
import { Search, X, Keyboard } from 'lucide-react';
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

export function PlateInput({
  value,
  onChange,
  onSubmit,
  isLoading,
  className,
  placeholder = 'Enter plate number',
}: Readonly<PlateInputProps>) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const [isFocused, setIsFocused] = useState(false);

  // Auto-format plate number (uppercase, common separators)
  const formatPlate = (input: string): string => {
    return input
      .toUpperCase()
      .replace(/[^A-Z0-9\-\s]/g, '')
      .slice(0, 12);
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(formatPlate(e.target.value));
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && value.length >= 4) {
      onSubmit();
    }
  };

  const handleClear = () => {
    onChange('');
    inputRef.current?.focus();
  };

  return (
    <div className={cn('relative', className)}>
      <div
        className={cn(
          'flex items-center gap-2 rounded-xl border-2 bg-card p-2 transition-all duration-200',
          isFocused
            ? 'border-accent ring-4 ring-accent/20'
            : 'border-border hover:border-muted-foreground/30'
        )}
      >
        {/* Plate icon/prefix */}
        <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground">
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
          onClick={onSubmit}
          disabled={value.length < 4 || isLoading}
          className="h-12 w-12 shrink-0 rounded-lg bg-accent text-accent-foreground hover:bg-accent/90"
        >
          {isLoading ? (
            <div className="h-5 w-5 animate-spin rounded-full border-2 border-current border-t-transparent" />
          ) : (
            <Search className="h-5 w-5" />
          )}
        </Button>
      </div>

      {/* Validation hint */}
      {value.length > 0 && value.length < 4 && (
        <p className="mt-2 text-center text-sm text-muted-foreground">
          {t('mobileSearch.helpTextShort', { count: 4 })}
        </p>
      )}
    </div>
  );
}
