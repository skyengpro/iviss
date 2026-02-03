import React from "react";
import { Button } from "@/components/ui/button";
import { Plus, CheckCircle } from "lucide-react";

interface VehicleActionFooterProps {
    controlLogged: boolean;
    isLoggingControl: boolean;
    onLogControl: () => void;
    onNewSearch: () => void;
}

export const VehicleActionFooter: React.FC<VehicleActionFooterProps> = ({
    controlLogged,
    isLoggingControl,
    onLogControl,
    onNewSearch,
}) => {
    return (
        <div className="fixed bottom-0 left-0 right-0 border-t border-border bg-background p-4 space-y-2 z-30">
            {!controlLogged ? (
                <Button
                    className="w-full h-12 gap-2 bg-accent text-accent-foreground hover:bg-accent/90"
                    onClick={onLogControl}
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
                onClick={onNewSearch}
            >
                New Search
            </Button>
        </div>
    );
};
