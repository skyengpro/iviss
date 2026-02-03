import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { StatusBadge } from '@/components/ui/status-badge';
import {
  ArrowLeft,
  Car,
  User,
  MapPin,
  Clock,
  Smartphone,
  FileText,
  CheckCircle,
  AlertTriangle,
  AlertCircle,
  Download,
} from 'lucide-react';
import { mockControlService, ControlRecord } from '@/services/mockControls';

export default function ControlDetail() {
  const { controlId } = useParams<{ controlId: string }>();
  const navigate = useNavigate();
  const [control, setControl] = useState<ControlRecord | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const loadControl = async () => {
      if (!controlId) return;

      try {
        const data = await mockControlService.getControlById(controlId);
        setControl(data);
      } catch (error) {
        console.error('Failed to load control:', error);
      } finally {
        setIsLoading(false);
      }
    };

    loadControl();
  }, [controlId]);

  if (isLoading) {
    return (
      <BackOfficeLayout title="Loading...">
        <div className="flex items-center justify-center py-12">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
      </BackOfficeLayout>
    );
  }

  if (!control) {
    return (
      <BackOfficeLayout title="Control Not Found">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Control not found</p>
          <Button className="mt-4" onClick={() => navigate('/backoffice/controls')}>
            Back to Controls
          </Button>
        </div>
      </BackOfficeLayout>
    );
  }

  const getStatusIcon = (status: string) => {
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

  return (
    <BackOfficeLayout
      title="Control Details"
      subtitle={`Control ID: ${control.id}`}
      actions={
        <div className="flex gap-2">
          <Button variant="outline" className="gap-2">
            <Download className="h-4 w-4" />
            Export PDF
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        <Button variant="ghost" className="gap-2" onClick={() => navigate('/backoffice/controls')}>
          <ArrowLeft className="h-4 w-4" />
          Back to Controls
        </Button>

        {/* Header Card */}
        <Card>
          <CardContent className="p-6">
            <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
              <div className="flex items-center gap-4">
                <div className="flex h-16 w-16 items-center justify-center rounded-xl bg-primary text-primary-foreground">
                  <Car className="h-8 w-8" />
                </div>
                <div>
                  <p className="text-3xl font-bold font-mono tracking-widest">
                    {control.plateNumber}
                  </p>
                  <p className="text-muted-foreground mt-1">
                    {control.identificationMode.toUpperCase()} identification
                    {control.confidence && ` • ${control.confidence}% confidence`}
                  </p>
                </div>
              </div>
              <StatusBadge
                variant={
                  control.status === 'valid'
                    ? 'valid'
                    : control.status === 'warning'
                      ? 'warning'
                      : 'critical'
                }
                size="lg"
              >
                {control.status.toUpperCase()}
              </StatusBadge>
            </div>
          </CardContent>
        </Card>

        <div className="grid gap-6 lg:grid-cols-2">
          {/* Control Information */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <FileText className="h-5 w-5 text-accent" />
                Control Information
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <InfoRow
                icon={Clock}
                label="Date & Time"
                value={control.timestamp.toLocaleString()}
              />
              <InfoRow icon={MapPin} label="Location" value={control.location.address} />
              <InfoRow icon={User} label="Agent" value={control.agentName} />
              <InfoRow icon={Smartphone} label="Device IMEI" value={control.phoneIMEI} />
            </CardContent>
          </Card>

          {/* Verification Results */}
          <Card>
            <CardHeader>
              <CardTitle>Verification Results</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <ResultRow
                label="Registration"
                status={control.results.registration}
                icon={getStatusIcon(control.results.registration)}
              />
              <ResultRow
                label="Insurance"
                status={control.results.insurance}
                icon={getStatusIcon(control.results.insurance)}
              />
              <ResultRow
                label="Technical Inspection"
                status={control.results.technicalInspection}
                icon={getStatusIcon(control.results.technicalInspection)}
              />
              <ResultRow
                label="Wanted Status"
                status={control.results.wantedStatus}
                icon={getStatusIcon(control.results.wantedStatus)}
              />
              <ResultRow
                label="Customs"
                status={control.results.customsStatus}
                icon={getStatusIcon(control.results.customsStatus)}
              />
            </CardContent>
          </Card>
        </div>

        {/* Actions Timeline */}
        <Card>
          <CardHeader>
            <CardTitle>Actions Timeline</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {control.actions.map((action, index) => (
                <div key={index} className="flex gap-4">
                  <div className="flex flex-col items-center">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-accent text-accent-foreground text-sm">
                      {index + 1}
                    </div>
                    {index < control.actions.length - 1 && (
                      <div className="w-0.5 flex-1 bg-border mt-2" />
                    )}
                  </div>
                  <div className="flex-1 pb-4">
                    <p className="font-medium capitalize">{action.type}</p>
                    <p className="text-sm text-muted-foreground">{action.description}</p>
                    <p className="text-xs text-muted-foreground mt-1">
                      {action.timestamp.toLocaleString()}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        {/* Notes */}
        {control.notes && (
          <Card>
            <CardHeader>
              <CardTitle>Agent Notes</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground">{control.notes}</p>
            </CardContent>
          </Card>
        )}
      </div>
    </BackOfficeLayout>
  );
}

function InfoRow({
  icon: Icon,
  label,
  value,
}: {
  icon: React.ElementType;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-3">
      <Icon className="h-5 w-5 text-muted-foreground" />
      <div>
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="font-medium">{value}</p>
      </div>
    </div>
  );
}

function ResultRow({
  label,
  status,
  icon,
}: {
  label: string;
  status: string;
  icon: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between rounded-lg bg-muted/50 p-3">
      <span className="font-medium">{label}</span>
      <div className="flex items-center gap-2">
        {icon}
        <StatusBadge
          variant={status === 'valid' ? 'valid' : status === 'warning' ? 'warning' : 'critical'}
          size="sm"
        >
          {status.toUpperCase()}
        </StatusBadge>
      </div>
    </div>
  );
}
