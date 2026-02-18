import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ChevronDown, ChevronUp, Image as ImageIcon } from 'lucide-react';
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from '@/components/ui/collapsible';
import { Button } from '@/components/ui/button';

interface VehicleImageCollapsibleProps {
    imageUrl: string;
}

export const VehicleImageCollapsible: React.FC<VehicleImageCollapsibleProps> = ({ imageUrl }) => {
    const { t } = useTranslation();
    const [isOpen, setIsOpen] = useState(true);

    return (
        <Collapsible
            open={isOpen}
            onOpenChange={setIsOpen}
            className="w-full space-y-2"
        >
            <div className="flex items-center justify-between">
                <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                    {t('vehicleResult.vehiclePhoto')}
                </h3>
                <CollapsibleTrigger asChild>
                    <Button variant="ghost" size="sm" className="w-9 p-0">
                        {isOpen ? (
                            <ChevronUp className="h-4 w-4" />
                        ) : (
                            <ChevronDown className="h-4 w-4" />
                        )}
                        <span className="sr-only">Toggle Photo</span>
                    </Button>
                </CollapsibleTrigger>
            </div>

            <CollapsibleContent className="space-y-2 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 overflow-hidden transition-all duration-300">
                <div className="relative aspect-video w-full overflow-hidden rounded-xl border border-border shadow-md bg-muted">
                    <img
                        src={imageUrl}
                        alt="Vehicle"
                        className="h-full w-full object-cover transition-all hover:scale-105"
                        onError={(e) => {
                            (e.target as HTMLImageElement).src = 'https://images.unsplash.com/photo-1492144534655-ae79c964c9d7?q=80&w=600&h=400&auto=format&fit=crop';
                        }}
                    />
                    <div className="absolute bottom-2 right-2 rounded-full bg-black/50 p-1.5 text-white backdrop-blur-sm">
                        <ImageIcon className="h-4 w-4" />
                    </div>
                </div>
            </CollapsibleContent>
        </Collapsible>
    );
};
