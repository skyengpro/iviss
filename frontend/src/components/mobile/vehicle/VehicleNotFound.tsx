import React from 'react';
import { AlertCircle, ArrowLeft, Camera } from 'lucide-react';
import { Link, useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/button';

interface VehicleNotFoundProps {
  plateNumber?: string;
}

export const VehicleNotFound: React.FC<VehicleNotFoundProps> = ({ plateNumber }) => {
  const navigate = useNavigate();

  return (
    <div className="p-4 space-y-6">
      <button
        onClick={() => navigate(-1)}
        className="flex items-center gap-2 text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="h-4 w-4" />
        Back
      </button>

      <div className="rounded-xl border border-status-warning/30 bg-status-warning/10 p-6 text-center animate-slide-up">
        <AlertCircle className="mx-auto h-16 w-16 text-status-warning" />
        <h2 className="mt-4 text-xl font-bold">Vehicle Not Found</h2>
        <p className="mt-2 text-muted-foreground">
          Plate <span className="font-mono font-semibold">{plateNumber}</span> is not in our
          database.
        </p>
      </div>

      <div className="rounded-xl border border-border bg-card p-4">
        <h3 className="font-semibold mb-2">What would you like to do?</h3>
        <p className="text-sm text-muted-foreground mb-4">
          You can capture the vehicle's carte grise to register it in the system.
        </p>

        <div className="space-y-2">
          <Link to={`/mobile/carte-grise?plate=${encodeURIComponent(plateNumber || '')}`}>
            <Button className="w-full gap-2 bg-accent text-accent-foreground hover:bg-accent/90">
              <Camera className="h-4 w-4" />
              Capture Carte Grise
            </Button>
          </Link>
          <Button variant="outline" className="w-full" onClick={() => navigate('/mobile/search')}>
            New Search
          </Button>
        </div>
      </div>
    </div>
  );
};
