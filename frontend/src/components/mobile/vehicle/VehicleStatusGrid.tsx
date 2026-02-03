import React from "react";
import { StatusBadge } from "@/components/ui/status-badge";
import { AlertCircle, CheckCircle, AlertTriangle, Clock } from "lucide-react";
import { AggregatedVehicleStatus } from "@/services/mockExternalAPIs";

interface VehicleStatusGridProps {
    apiStatus: AggregatedVehicleStatus;
}

export const VehicleStatusGrid: React.FC<VehicleStatusGridProps> = ({ apiStatus }) => {
    return (
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
    );
};

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
