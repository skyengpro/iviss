import { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { PlateInput } from '@/components/vehicle/PlateInput';
import { Button } from '@/components/ui/button';
import { Camera, History } from 'lucide-react';
import { Link } from 'react-router-dom';

import { useQuery } from '@tanstack/react-query';
import { useAuth } from '@/contexts/AuthContext';
import { mockControlService } from '@/services/mockControls';

export default function MobileSearch() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [plateNumber, setPlateNumber] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  // Fetch recent controls for the agent to show as "recent searches"
  const { data: recentControls = [] } = useQuery({
    queryKey: ['recent-controls', user?.id],
    queryFn: () => (user ? mockControlService.getControlsByAgent(user.id, 5) : Promise.resolve([])),
    enabled: !!user,
  });

  // Check if plate was passed from scan
  useEffect(() => {
    const plateFromScan = searchParams.get('plate');
    if (plateFromScan) {
      setPlateNumber(plateFromScan);
      // Auto-search
      handleSearch(plateFromScan);
    }
  }, [searchParams]);

  const handleSearch = async (plate?: string) => {
    const searchPlate = plate || plateNumber;
    if (searchPlate.length < 4) return;

    setIsLoading(true);

    // Navigate to result page
    navigate(`/mobile/vehicle/${encodeURIComponent(searchPlate)}`);
  };

  const recentSearches = ['AB-123-CD', 'XY-789-ZW', 'EF-456-GH'];

  return (
    <MobileLayout title="Search Vehicle">
      <div className="p-4 space-y-6">
        {/* Input section */}
        <section>
          <PlateInput
            value={plateNumber}
            onChange={setPlateNumber}
            onSubmit={() => handleSearch()}
            isLoading={isLoading}
            placeholder="Enter plate number"
          />

          {/* Alternative actions */}
          <div className="mt-4 flex gap-2">
            <Link to="/mobile/scan?mode=photo" className="flex-1">
              <Button variant="outline" className="w-full gap-2">
                <Camera className="h-4 w-4" />
                Photo Scan
              </Button>
            </Link>
            <Link to="/mobile/history" className="flex-1">
              <Button variant="outline" className="w-full gap-2">
                <History className="h-4 w-4" />
                History
              </Button>
            </Link>
          </div>
        </section>

        {/* Recent searches */}
        <section>
          <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            Recent Activity
          </h3>
          <div className="space-y-2">
            {recentControls.length > 0 ? (
              recentControls.map((control) => (
                <button
                  key={control.id}
                  onClick={() => {
                    setPlateNumber(control.plateNumber);
                    handleSearch(control.plateNumber);
                  }}
                  className="flex w-full items-center justify-between rounded-lg border border-border bg-card p-3 text-left transition-colors hover:bg-muted active:scale-[0.98]"
                >
                  <span className="font-mono font-semibold tracking-wider">
                    {control.plateNumber}
                  </span>
                  <span className="text-xs text-muted-foreground">Tap to search</span>
                </button>
              ))
            ) : (
              <p className="py-4 text-center text-sm text-muted-foreground">
                No recent activity found.
              </p>
            )}
          </div>
        </section>

        {/* Help text */}
        <div className="rounded-lg bg-muted/50 p-4 text-center">
          <p className="text-sm text-muted-foreground">
            Enter at least 4 characters of the plate number to search. The system will query all
            national databases.
          </p>
        </div>
      </div>
    </MobileLayout>
  );
}
