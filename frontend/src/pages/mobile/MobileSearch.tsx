import { useState, useEffect, useCallback } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { PlateInput, isValidPlate } from '@/components/vehicle/PlateInput';
import { Button } from '@/components/ui/button';
import { Camera, History, Search } from 'lucide-react';

import { useAuth } from '@/hooks/auth/use-auth';
import { useControls } from '@/hooks/api/useControls';

export default function MobileSearch() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const { t } = useTranslation();
  const [searchParams] = useSearchParams();
  const [plateNumber, setPlateNumber] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleSearch = useCallback(
    async (plate?: string) => {
      const searchPlate = plate || plateNumber;

      setIsLoading(true);

      // Navigate to result page
      navigate(`/mobile/vehicle/${encodeURIComponent(searchPlate)}`);
    },
    [navigate, plateNumber]
  );

  // Fetch recent controls for the agent to show as "recent searches"
  const { controls: recentControls = [] } = useControls({
    query: {
      agent_id: user?.id || null,
      start_date: null,
      end_date: null,
      status: null,
      plate: null,
    },
  });

  // Check if plate was passed from scan
  useEffect(() => {
    const plateFromScan = searchParams.get('plate');
    if (plateFromScan) {
      setPlateNumber(plateFromScan);
      // Auto-search
      handleSearch(plateFromScan);
    }
  }, [searchParams, handleSearch]);

  return (
    <MobileLayout title={t('mobileSearch.title')}>
      <div className="p-4 space-y-6">
        {/* Input section */}
        <section>
          <PlateInput
            value={plateNumber}
            onChange={setPlateNumber}
            onSubmit={() => handleSearch()}
            isLoading={isLoading}
          />

          {/* Primary Search Button */}
          <Button
            onClick={() => handleSearch()}
            disabled={!isValidPlate(plateNumber) || isLoading}
            className="mt-4 w-full h-14 text-lg font-bold gap-3 rounded-xl shadow-lg active:scale-[0.98] transition-all"
          >
            {isLoading ? (
              <div className="h-6 w-6 animate-spin rounded-full border-2 border-white border-t-transparent" />
            ) : (
              <Search className="h-6 w-6" />
            )}
            {t('mobileSearch.searchButton')}
          </Button>

          {/* Alternative actions */}
          <div className="mt-4 flex gap-2">
            <Link to="/mobile/scan?mode=photo" className="flex-1">
              <Button variant="outline" className="w-full gap-2">
                <Camera className="h-4 w-4" />
                {t('mobileSearch.photoScan')}
              </Button>
            </Link>
            <Link to="/mobile/history" className="flex-1">
              <Button variant="outline" className="w-full gap-2">
                <History className="h-4 w-4" />
                {t('mobileSearch.history')}
              </Button>
            </Link>
          </div>
        </section>

        {/* Recent searches */}
        <section>
          <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            {t('mobileSearch.recentActivity')}
          </h3>
          <div className="space-y-2">
            {recentControls.length > 0 ? (
              recentControls.slice(0, 5).map((control) => (
                <button
                  key={control.id}
                  onClick={() => {
                    setPlateNumber(control.plate_number);
                    handleSearch(control.plate_number);
                  }}
                  className="flex w-full items-center justify-between rounded-lg border border-border bg-card p-3 text-left transition-colors hover:bg-muted active:scale-[0.98]"
                >
                  <span className="font-mono font-semibold tracking-wider">
                    {control.plate_number}
                  </span>
                  <span className="text-xs text-muted-foreground">
                    {t('mobileSearch.tapToSearch')}
                  </span>
                </button>
              ))
            ) : (
              <p className="py-4 text-center text-sm text-muted-foreground">
                {t('mobileSearch.noRecentActivity')}
              </p>
            )}
          </div>
        </section>

        {/* Help text */}
        <div className="rounded-lg bg-muted/50 p-4 text-center">
          <p className="text-sm text-muted-foreground">{t('mobileSearch.helpText')}</p>
        </div>
      </div>
    </MobileLayout>
  );
}
