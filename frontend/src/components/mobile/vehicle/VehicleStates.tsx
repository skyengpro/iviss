import React from 'react';
import { Shield, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';

export const VehicleLoadingState: React.FC<{ queryTime?: number }> = ({ queryTime }) => (
  <div className="flex flex-col items-center justify-center min-h-[60vh] p-4">
    <div className="relative">
      <div className="h-16 w-16 animate-spin rounded-full border-4 border-muted border-t-accent" />
      <Shield className="absolute inset-0 m-auto h-6 w-6 text-accent" />
    </div>
    <p className="mt-6 text-lg font-medium">Searching databases...</p>
    <p className="mt-2 text-sm text-muted-foreground text-center">
      Querying Insurance, Police, Customs, and Technical Inspection systems
    </p>
    {queryTime && <p className="mt-4 text-xs text-muted-foreground">Query time: {queryTime}ms</p>}
  </div>
);

export const VehicleErrorState: React.FC<{ onRetry: () => void }> = ({ onRetry }) => (
  <div className="flex flex-col items-center justify-center min-h-[60vh] p-4">
    <AlertCircle className="h-16 w-16 text-destructive" />
    <h2 className="mt-4 text-xl font-bold">Search Error</h2>
    <p className="mt-2 text-muted-foreground text-center">
      An error occurred while searching. Please try again.
    </p>
    <Button className="mt-6" onClick={onRetry}>
      Try Again
    </Button>
  </div>
);
