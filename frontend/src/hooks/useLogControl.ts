import { useState } from "react";
import { mockControlService } from "@/services/mockControls";
import { toast } from "@/hooks/use-toast";
import { User } from "@/services/mockAuth";
import { Vehicle } from "@/services/mockVehicles";
import { AggregatedVehicleStatus } from "@/services/mockExternalAPIs";

export function useLogControl() {
    const [isLoggingControl, setIsLoggingControl] = useState(false);
    const [controlLogged, setControlLogged] = useState(false);

    const logControl = async (
        user: User,
        plateNumber: string,
        apiStatus: AggregatedVehicleStatus,
        vehicle: Vehicle | null
    ) => {
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
            return true;
        } catch (error) {
            toast({
                title: "Error",
                description: "Failed to log control. Please try again.",
                variant: "destructive",
            });
            return false;
        } finally {
            setIsLoggingControl(false);
        }
    };

    return {
        isLoggingControl,
        controlLogged,
        logControl,
        setControlLogged
    };
}
