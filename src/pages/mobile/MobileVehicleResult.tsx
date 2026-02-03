import { useState, useEffect } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { MobileLayout } from "@/components/layout/MobileLayout";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/ui/status-badge";
import { 
  AlertCircle, 
  CheckCircle, 
  AlertTriangle, 
  Shield, 
  Car, 
  User, 
  FileText,
  Clock,
  ArrowLeft,
  Plus,
  Camera
} from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { mockVehicleService, Vehicle } from "@/services/mockVehicles";
import { mockExternalAPIService, AggregatedVehicleStatus } from "@/services/mockExternalAPIs";
import { mockControlService } from "@/services/mockControls";
import { toast } from "@/hooks/use-toast";

type LoadingState = 'loading' | 'found' | 'not-found' | 'error';

export default function MobileVehicleResult() {
  const { plateNumber } = useParams<{ plateNumber: string }>();
  const navigate = useNavigate();
  const { user } = useAuth();
  
  const [loadingState, setLoadingState] = useState<LoadingState>('loading');
  const [vehicle, setVehicle] = useState<Vehicle | null>(null);
  const [apiStatus, setApiStatus] = useState<AggregatedVehicleStatus | null>(null);
  const [isLoggingControl, setIsLoggingControl] = useState(false);
  const [controlLogged, setControlLogged] = useState(false);

  useEffect(() => {
    const searchVehicle = async () => {
      if (!plateNumber) {
        setLoadingState('error');
        return;
      }

      setLoadingState('loading');

      try {
        // Search vehicle in database
        const vehicleResult = await mockVehicleService.searchByPlate(plateNumber);
        
        // Query external APIs
        const apiResult = await mockExternalAPIService.checkAllSystems(plateNumber);
        setApiStatus(apiResult);

        if (vehicleResult.found && vehicleResult.vehicle) {
          setVehicle(vehicleResult.vehicle);
          setLoadingState('found');
        } else {
          setLoadingState('not-found');
        }
      } catch (error) {
        console.error('Search error:', error);
        setLoadingState('error');
      }
    };

    searchVehicle();
  }, [plateNumber]);

  const handleLogControl = async () => {
    if (!user || !plateNumber || !apiStatus) return;

    setIsLoggingControl(true);

    try {
      // Map API status to control status (handle 'unknown' as 'pending')
      const mapStatus = (status: string): 'valid' | 'warning' | 'critical' | 'pending' => {
        if (status === 'unknown') return 'pending';
        return status as 'valid' | 'warning' | 'critical' | 'pending';
      };

      await mockControlService.logControl({
        plateNumber: plateNumber,
        vehicleId: vehicle?.id,
        agentId: user.id,
        agentName: user.name,
        organizationId: user.organizationId,
        organizationName: user.organization,
        phoneIMEI: user.phoneIMEI,
        location: {
          address: 'Highway A1, KM 42',
          latitude: 48.8566,
          longitude: 2.3522,
        },
        identificationMode: 'manual',
        results: {
          registration: vehicle?.registration.status || 'valid',
          insurance: mapStatus(apiStatus.insurance.status),
          technicalInspection: mapStatus(apiStatus.technicalInspection.status),
          wantedStatus: mapStatus(apiStatus.police.status),
          customsStatus: mapStatus(apiStatus.customs.status),
        },
      });

      setControlLogged(true);
      toast({
        title: "Control Logged",
        description: "The control has been successfully recorded.",
      });
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to log control. Please try again.",
        variant: "destructive",
      });
    } finally {
      setIsLoggingControl(false);
    }
  };

  if (loadingState === 'loading') {
    return (
      <MobileLayout title="Searching..." hideNavigation>
        <div className="flex flex-col items-center justify-center min-h-[60vh] p-4">
          <div className="relative">
            <div className="h-16 w-16 animate-spin rounded-full border-4 border-muted border-t-accent" />
            <Shield className="absolute inset-0 m-auto h-6 w-6 text-accent" />
          </div>
          <p className="mt-6 text-lg font-medium">Searching databases...</p>
          <p className="mt-2 text-sm text-muted-foreground text-center">
            Querying Insurance, Police, Customs, and Technical Inspection systems
          </p>
          {apiStatus && (
            <p className="mt-4 text-xs text-muted-foreground">
              Query time: {apiStatus.queryTime}ms
            </p>
          )}
        </div>
      </MobileLayout>
    );
  }

  if (loadingState === 'not-found') {
    return (
      <MobileLayout title="Vehicle Not Found" hideNavigation>
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
              Plate <span className="font-mono font-semibold">{plateNumber}</span> is not in our database.
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
              <Button 
                variant="outline" 
                className="w-full"
                onClick={() => navigate('/mobile/search')}
              >
                New Search
              </Button>
            </div>
          </div>
        </div>
      </MobileLayout>
    );
  }

  if (loadingState === 'error' || !vehicle || !apiStatus) {
    return (
      <MobileLayout title="Error" hideNavigation>
        <div className="flex flex-col items-center justify-center min-h-[60vh] p-4">
          <AlertCircle className="h-16 w-16 text-destructive" />
          <h2 className="mt-4 text-xl font-bold">Search Error</h2>
          <p className="mt-2 text-muted-foreground text-center">
            An error occurred while searching. Please try again.
          </p>
          <Button 
            className="mt-6"
            onClick={() => navigate('/mobile/search')}
          >
            Try Again
          </Button>
        </div>
      </MobileLayout>
    );
  }

  // Determine overall alert status
  const isCritical = apiStatus.overallStatus === 'critical';
  const isWarning = apiStatus.overallStatus === 'warning';

  return (
    <MobileLayout title="Vehicle Details" hideNavigation>
      <div className="p-4 space-y-4 pb-32">
        <button
          onClick={() => navigate(-1)}
          className="flex items-center gap-2 text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </button>

        {/* Alert Banner */}
        {isCritical && (
          <div className="rounded-xl bg-status-critical p-4 text-status-critical-foreground animate-pulse-status">
            <div className="flex items-center gap-3">
              <AlertTriangle className="h-6 w-6" />
              <div>
                <p className="font-bold">CRITICAL ALERT</p>
                <p className="text-sm opacity-90">
                  {apiStatus.police.isStolen ? 'STOLEN VEHICLE' : 
                   apiStatus.police.isWanted ? 'WANTED VEHICLE' : 
                   'Immediate attention required'}
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Vehicle Info Card */}
        <div className="rounded-xl border border-border bg-card overflow-hidden animate-slide-up">
          <div className="bg-primary p-4 text-primary-foreground">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-xs opacity-70">Plate Number</p>
                <p className="text-2xl font-bold tracking-widest font-mono">{vehicle.plateNumber}</p>
              </div>
              <StatusBadge 
                variant={isCritical ? 'critical' : isWarning ? 'warning' : 'valid'}
                size="lg"
              >
                {isCritical ? 'CRITICAL' : isWarning ? 'WARNING' : 'VALID'}
              </StatusBadge>
            </div>
          </div>

          <div className="p-4 space-y-4">
            {/* Vehicle Details */}
            <div className="grid grid-cols-2 gap-4">
              <DetailItem icon={Car} label="Brand" value={vehicle.brand} />
              <DetailItem icon={Car} label="Model" value={vehicle.model} />
              <DetailItem icon={FileText} label="Year" value={String(vehicle.year)} />
              <DetailItem icon={FileText} label="Power" value={vehicle.enginePower} />
            </div>

            <div className="border-t border-border pt-4">
              <DetailItem 
                icon={FileText} 
                label="Chassis Number" 
                value={vehicle.chassisNumber} 
                fullWidth
              />
            </div>

            <div className="border-t border-border pt-4">
              <DetailItem 
                icon={User} 
                label="Owner" 
                value={vehicle.owner.name}
                fullWidth
              />
              <p className="text-sm text-muted-foreground mt-1 ml-8">
                {vehicle.owner.address}
              </p>
            </div>
          </div>
        </div>

        {/* Status Cards */}
        <div className="space-y-3">
          <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            Legal Status
          </h3>

          <StatusCard
            title="Insurance"
            status={apiStatus.insurance.status}
            provider={apiStatus.insurance.provider}
            expiryDate={apiStatus.insurance.expiryDate}
            notes={apiStatus.insurance.notes}
          />

          <StatusCard
            title="Technical Inspection"
            status={apiStatus.technicalInspection.status}
            expiryDate={apiStatus.technicalInspection.expiryDate}
            notes={apiStatus.technicalInspection.notes}
          />

          <StatusCard
            title="Wanted Status"
            status={apiStatus.police.status}
            notes={apiStatus.police.notes}
            isAlert={apiStatus.police.isWanted || apiStatus.police.isStolen}
          />

          <StatusCard
            title="Customs Clearance"
            status={apiStatus.customs.status}
            notes={apiStatus.customs.notes}
          />
        </div>

        {/* Query Time */}
        <div className="flex items-center justify-center gap-2 text-xs text-muted-foreground">
          <Clock className="h-3 w-3" />
          <span>Query completed in {apiStatus.queryTime}ms</span>
        </div>
      </div>

      {/* Bottom Actions */}
      <div className="fixed bottom-0 left-0 right-0 border-t border-border bg-background p-4 space-y-2">
        {!controlLogged ? (
          <Button
            className="w-full h-12 gap-2 bg-accent text-accent-foreground hover:bg-accent/90"
            onClick={handleLogControl}
            disabled={isLoggingControl}
          >
            {isLoggingControl ? (
              <div className="h-5 w-5 animate-spin rounded-full border-2 border-current border-t-transparent" />
            ) : (
              <>
                <Plus className="h-5 w-5" />
                Log Control
              </>
            )}
          </Button>
        ) : (
          <Button
            className="w-full h-12 gap-2 bg-status-valid text-status-valid-foreground"
            disabled
          >
            <CheckCircle className="h-5 w-5" />
            Control Logged
          </Button>
        )}
        <Button
          variant="outline"
          className="w-full"
          onClick={() => navigate('/mobile/search')}
        >
          New Search
        </Button>
      </div>
    </MobileLayout>
  );
}

function DetailItem({ 
  icon: Icon, 
  label, 
  value, 
  fullWidth 
}: { 
  icon: React.ElementType; 
  label: string; 
  value: string;
  fullWidth?: boolean;
}) {
  return (
    <div className={`flex items-start gap-2 ${fullWidth ? '' : ''}`}>
      <Icon className="h-4 w-4 text-muted-foreground mt-0.5" />
      <div>
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="font-medium">{value}</p>
      </div>
    </div>
  );
}

function StatusCard({
  title,
  status,
  provider,
  expiryDate,
  notes,
  isAlert,
}: {
  title: string;
  status: 'valid' | 'warning' | 'critical' | 'unknown';
  provider?: string;
  expiryDate?: string;
  notes?: string;
  isAlert?: boolean;
}) {
  const getStatusIcon = () => {
    switch (status) {
      case 'valid':
        return <CheckCircle className="h-5 w-5 text-status-valid" />;
      case 'warning':
        return <AlertTriangle className="h-5 w-5 text-status-warning" />;
      case 'critical':
        return <AlertCircle className="h-5 w-5 text-status-critical" />;
      default:
        return <Clock className="h-5 w-5 text-muted-foreground" />;
    }
  };

  const getBorderColor = () => {
    switch (status) {
      case 'valid':
        return 'border-status-valid/30';
      case 'warning':
        return 'border-status-warning/30';
      case 'critical':
        return 'border-status-critical/30';
      default:
        return 'border-border';
    }
  };

  const getBgColor = () => {
    switch (status) {
      case 'critical':
        return 'bg-status-critical/5';
      case 'warning':
        return 'bg-status-warning/5';
      default:
        return 'bg-card';
    }
  };

  return (
    <div className={`rounded-xl border ${getBorderColor()} ${getBgColor()} p-4 ${isAlert ? 'animate-pulse-status' : ''}`}>
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          {getStatusIcon()}
          <div>
            <p className="font-semibold">{title}</p>
            {provider && <p className="text-sm text-muted-foreground">{provider}</p>}
            {expiryDate && <p className="text-sm text-muted-foreground">Expires: {expiryDate}</p>}
          </div>
        </div>
        <StatusBadge variant={status === 'unknown' ? 'pending' : status} size="sm">
          {status.toUpperCase()}
        </StatusBadge>
      </div>
      {notes && (
        <p className={`mt-2 text-sm ${status === 'critical' ? 'text-status-critical font-medium' : 'text-muted-foreground'}`}>
          {notes}
        </p>
      )}
    </div>
  );
}
