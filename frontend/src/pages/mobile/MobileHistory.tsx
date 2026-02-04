import { MobileLayout } from "@/components/layout/MobileLayout";
import { StatusBadge } from "@/components/ui/status-badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Search,
  Filter,
  Calendar,
  MapPin,
  Clock,
  ChevronRight,
  Download
} from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { mockControlService, ControlRecord, Translatable } from "@/services/mockControls";
import { useAuth } from "@/contexts/AuthContext";

type FilterStatus = 'all' | 'valid' | 'warning' | 'critical';

export default function MobileHistory() {
  const { user } = useAuth();
  const { t, i18n } = useTranslation();
  const [searchQuery, setSearchQuery] = useState("");
  const [filterStatus, setFilterStatus] = useState<FilterStatus>("all");

  const { data: controls = [], isLoading } = useQuery<ControlRecord[]>({
    queryKey: ['controls', user?.id],
    queryFn: () => (user ? mockControlService.getControlsByAgent(user.id) : Promise.resolve([])),
    enabled: !!user,
  });

    const renderNotes = (notes: Translatable): string => {
    if (!notes) return '';
    if (typeof notes === 'string') {
      return notes;
    }
    return t(notes.key, notes.params);
  };

  const filteredControls = controls.filter((control) => {
    const noteString = renderNotes(control.notes);
    const matchesSearch =
      control.plateNumber.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (noteString && noteString.toLowerCase().includes(searchQuery.toLowerCase()));

    const matchesFilter = filterStatus === 'all' || control.status === filterStatus;

    return matchesSearch && matchesFilter;
  });

  const formatTime = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleTimeString(i18n.language, {
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const formatDate = (isoString: string) => {
    const date = new Date(isoString);
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    if (date.toDateString() === today.toDateString()) {
      return t('mobileHistory.today');
    } else if (date.toDateString() === yesterday.toDateString()) {
      return t('mobileHistory.yesterday');
    }
    return date.toLocaleDateString(i18n.language, {
      month: "short",
      day: "numeric",
    });
  };

  // Group controls by date
  const groupedControls = filteredControls.reduce(
    (groups, control) => {
      const dateKey = formatDate(control.timestamp.toISOString());
      if (!groups[dateKey]) {
        groups[dateKey] = [];
      }
      groups[dateKey].push(control);
      return groups;
    },
    {} as Record<string, ControlRecord[]>
  );

  return (
    <MobileLayout title={t('mobileHistory.title')}>
      <div className="p-4 space-y-4">
        {isLoading && (
          <div className="space-y-4">
            <div className="h-10 bg-muted animate-pulse rounded-md" />
            <div className="space-y-2">
              {[1, 2, 3].map((i) => (
                <div key={i} className="h-24 bg-muted animate-pulse rounded-xl" />
              ))}
            </div>
          </div>
        )}

        {!isLoading && (
          <>
            {/* Search and filters */}
            <div className="flex gap-2">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder={t('mobileHistory.searchPlaceholder')}
                  className="pl-9"
                />
              </div>
              <Button variant="outline" size="icon">
                <Filter className="h-4 w-4" />
              </Button>
            </div>

            {/* Status filter pills */}
            <div className="flex gap-2 overflow-x-auto pb-2">
              {(["all", "valid", "warning", "critical"] as FilterStatus[]).map(
                (status) => (
                  <button
                    key={status}
                    onClick={() => setFilterStatus(status)}
                    className={cn(
                      "shrink-0 rounded-full px-4 py-2 text-sm font-medium transition-colors",
                      filterStatus === status
                        ? status === "all"
                          ? "bg-primary text-primary-foreground"
                          : status === "valid"
                            ? "bg-status-valid text-status-valid-foreground"
                            : status === "warning"
                              ? "bg-status-warning text-status-warning-foreground"
                              : "bg-status-critical text-status-critical-foreground"
                        : "bg-muted text-muted-foreground hover:bg-muted/80"
                    )}
                  >
                    {t(`mobileHistory.${status}`)}
                  </button>
                )
              )}
            </div>

            {/* Summary */}
            <div className="rounded-lg bg-muted p-3 text-center">
              <p className="text-sm text-muted-foreground">
                {t('mobileHistory.showingControls', { count: filteredControls.length })}
              </p>
            </div>

            {/* Controls list grouped by date */}
            <div className="space-y-6">
              {Object.entries(groupedControls).map(([date, controls]) => (
                <div key={date}>
                  <div className="mb-2 flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm font-semibold text-muted-foreground">{date}</span>
                  </div>
                  <div className="space-y-2">
                    {controls.map((control) => (
                      <ControlItem key={control.id} control={control} formatTime={formatTime} renderNotes={renderNotes} />
                    ))}
                  </div>
                </div>
              ))}
            </div>

            {/* Export button */}
            <div className="pt-4">
              <Button variant="outline" className="w-full gap-2">
                <Download className="h-4 w-4" />
                {t('mobileHistory.export')}
              </Button>
            </div>
          </>
        )}
      </div>
    </MobileLayout>
  );
}

function ControlItem({
  control,
  formatTime,
  renderNotes,
}: {
    control: ControlRecord;
  formatTime: (isoString: string) => string;
  renderNotes: (notes: Translatable) => string;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-3 rounded-xl border border-border bg-card p-4 transition-colors active:bg-muted">
      {/* Status indicator */}
      <div
        className={cn(
          'h-12 w-1 rounded-full',
          control.status === 'valid' && 'bg-status-valid',
          control.status === 'warning' && 'bg-status-warning',
          control.status === 'critical' && 'bg-status-critical'
        )}
      />

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between">
          <p className="font-mono text-lg font-bold tracking-wider">{control.plateNumber}</p>
          <StatusBadge variant={control.status} size="sm" showIcon={false}>
            {t(`mobileHistory.${control.status}`)}
          </StatusBadge>
        </div>
                {control.notes && (
          <p className="mt-1 text-sm text-muted-foreground truncate">
            {renderNotes(control.notes)}
          </p>
        )}
        <div className="mt-2 flex items-center gap-4 text-xs text-muted-foreground">
          <div className="flex items-center gap-1">
            <Clock className="h-3 w-3" />
            <span>{formatTime(control.timestamp.toISOString())}</span>
          </div>
          <div className="flex items-center gap-1 truncate">
            <MapPin className="h-3 w-3 shrink-0" />
            <span className="truncate">{control.location.address}</span>
          </div>
        </div>
      </div>

      <ChevronRight className="h-5 w-5 shrink-0 text-muted-foreground" />
    </div>
  );
}
