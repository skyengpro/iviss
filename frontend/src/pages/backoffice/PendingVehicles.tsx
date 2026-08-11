import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { StatusBadge } from '@/components/ui/status-badge';
import {
  CheckCircle,
  XCircle,
  FileText,
  User,
  MapPin,
  Clock,
  Eye,
  AlertCircle,
  ChevronRight,
  History,
  X,
} from 'lucide-react';
import { toast } from '@/hooks/ui/use-toast';
import {
  getSubmissions,
  getSubmissionById,
  getSubmissionAuditLog,
  type PendingSubmissionListItem,
  type PendingSubmissionDetail,
  type SubmissionAuditLogEntry,
  type SubmissionStatus,
} from '@/services/api/submissionService';

// ── Status filter tabs ──────────────────────────────────────────────────────

type FilterTab = 'all' | SubmissionStatus;

const FILTER_TABS: { key: FilterTab; labelKey: string }[] = [
  { key: 'all', labelKey: 'pendingValidation.filterAll' },
  { key: 'pending', labelKey: 'pendingValidation.filterPending' },
  { key: 'approved', labelKey: 'pendingValidation.filterApproved' },
  { key: 'rejected', labelKey: 'pendingValidation.filterRejected' },
];

const statusVariantMap: Record<SubmissionStatus, 'pending' | 'valid' | 'critical'> = {
  pending: 'pending',
  approved: 'valid',
  rejected: 'critical',
};

// ── Vehicle data entry form defaults ────────────────────────────────────────

// ── Main component ──────────────────────────────────────────────────────────

export default function PendingVehicles() {
  const { t } = useTranslation();

  // List state
  const [submissions, setSubmissions] = useState<PendingSubmissionListItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [activeFilter, setActiveFilter] = useState<FilterTab>('pending');

  // Detail state
  const [selectedDetail, setSelectedDetail] = useState<PendingSubmissionDetail | null>(null);
  const [isLoadingDetail, setIsLoadingDetail] = useState(false);

  // Audit log
  const [auditLog, setAuditLog] = useState<SubmissionAuditLogEntry[]>([]);
  const [showAuditLog, setShowAuditLog] = useState(false);

  // ── Data Loading ──────────────────────────────────────────────────────

  const loadSubmissions = useCallback(async () => {
    setIsLoading(true);
    try {
      const filter = activeFilter === 'all' ? undefined : activeFilter;
      const data = await getSubmissions(filter);
      setSubmissions(data);
    } catch (error) {
      console.error('Failed to load submissions:', error);
      toast({
        title: t('pendingValidation.toastError'),
        description: t('pendingValidation.loadError'),
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  }, [activeFilter, t]);

  useEffect(() => {
    loadSubmissions();
  }, [loadSubmissions]);

  const loadDetail = async (id: string) => {
    setIsLoadingDetail(true);
    setShowAuditLog(false);
    try {
      const detail = await getSubmissionById(id);
      setSelectedDetail(detail);
    } catch (error) {
      console.error('Failed to load submission detail:', error);
      toast({
        title: t('pendingValidation.toastError'),
        description: t('pendingValidation.detailLoadError'),
        variant: 'destructive',
      });
    } finally {
      setIsLoadingDetail(false);
    }
  };

  const loadAuditLog = async (id: string) => {
    try {
      const log = await getSubmissionAuditLog(id);
      setAuditLog(log);
      setShowAuditLog(true);
    } catch (error) {
      console.error('Failed to load audit log:', error);
    }
  };

  // ── Helpers ───────────────────────────────────────────────────────────

  const formatDate = (isoString: string) => {
    try {
      const date = new Date(isoString);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
      if (diffHours < 1) return t('pendingValidation.lessThanHourAgo');
      if (diffHours < 24) return t('pendingValidation.hoursAgo', { count: diffHours });
      return date.toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return isoString;
    }
  };

  const pendingCount = submissions.filter((s) => s.status === 'pending').length;

  // ── Render ────────────────────────────────────────────────────────────

  return (
    <BackOfficeLayout
      title={t('pendingValidation.title')}
      subtitle={t('pendingValidation.subtitle', { count: pendingCount })}
    >
      <div className="grid gap-6 lg:grid-cols-3">
        {/* ── Left column: List + Filters ─────────────────────────── */}
        <div className="lg:col-span-1 space-y-4">
          {/* Filter tabs */}
          <div className="flex rounded-lg border border-border bg-muted/30 p-1">
            {FILTER_TABS.map((tab) => (
              <button
                key={tab.key}
                onClick={() => setActiveFilter(tab.key)}
                className={`flex-1 rounded-md px-2 py-1.5 text-xs font-semibold transition-all ${
                  activeFilter === tab.key
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {t(tab.labelKey)}
              </button>
            ))}
          </div>

          {/* Submissions list */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-base">
                <AlertCircle className="h-5 w-5 text-status-warning" />
                {t('pendingValidation.submissions')}
                <span className="ml-auto rounded-full bg-muted px-2 py-0.5 text-xs font-medium">
                  {submissions.length}
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <div className="flex items-center justify-center py-8">
                  <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                </div>
              ) : submissions.length === 0 ? (
                <div className="text-center py-8">
                  <CheckCircle className="mx-auto h-12 w-12 text-status-valid" />
                  <p className="mt-4 text-muted-foreground">
                    {t('pendingValidation.noSubmissions')}
                  </p>
                </div>
              ) : (
                <div className="space-y-2 max-h-[calc(100vh-20rem)] overflow-y-auto">
                  {submissions.map((sub) => {
                    const hasDetail = sub.source === 'submission' && sub.id !== null;
                    return (
                      <button
                        key={sub.id ?? sub.plateNumber}
                        onClick={hasDetail ? () => loadDetail(sub.id as string) : undefined}
                        disabled={!hasDetail}
                        title={hasDetail ? undefined : t('pendingValidation.noDetailAvailable')}
                        className={`w-full text-left rounded-lg border p-3 transition-colors ${
                          !hasDetail
                            ? 'border-border opacity-70 cursor-not-allowed'
                            : selectedDetail?.id === sub.id
                              ? 'border-accent bg-accent/5'
                              : 'border-border hover:bg-muted'
                        }`}
                      >
                        <div className="flex items-center justify-between">
                          <span className="font-mono font-bold tracking-wider text-sm">
                            {sub.plateNumber}
                          </span>
                          {hasDetail ? (
                            <StatusBadge variant={statusVariantMap[sub.status]} size="sm">
                              {t(`pendingValidation.status_${sub.status}`)}
                            </StatusBadge>
                          ) : (
                            <StatusBadge variant="pending" size="sm">
                              {t('pendingValidation.unregisteredSource')}
                            </StatusBadge>
                          )}
                        </div>
                        <div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
                          <User className="h-3 w-3" />
                          <span>{sub.agentName ?? t('pendingValidation.unknownAgent')}</span>
                        </div>
                        <div className="mt-1 flex items-center justify-between text-xs text-muted-foreground">
                          <span className="flex items-center gap-1.5">
                            <Clock className="h-3 w-3" />
                            {formatDate(sub.submittedAt)}
                          </span>
                          {hasDetail && <ChevronRight className="h-3.5 w-3.5" />}
                        </div>
                      </button>
                    );
                  })}
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* ── Right column: Detail / Forms ────────────────────────── */}
        <div className="lg:col-span-2 space-y-4">
          {isLoadingDetail ? (
            <Card>
              <CardContent className="flex items-center justify-center py-16">
                <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
              </CardContent>
            </Card>
          ) : selectedDetail ? (
            <>
              {/* ── Detail card ─────────────────────────────────── */}
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="flex items-center gap-2">
                      <FileText className="h-5 w-5 text-accent" />
                      {t('pendingValidation.reviewSubmission')}
                    </CardTitle>
                    <StatusBadge variant={statusVariantMap[selectedDetail.status]} size="lg">
                      {t(`pendingValidation.status_${selectedDetail.status}`).toUpperCase()}
                    </StatusBadge>
                  </div>
                </CardHeader>
                <CardContent className="space-y-6">
                  {/* Plate number */}
                  <div className="rounded-lg bg-muted p-4">
                    <p className="text-3xl font-bold font-mono tracking-widest">
                      {selectedDetail.plateNumber}
                    </p>
                  </div>

                  {/* Submission metadata */}
                  <div className="grid gap-4 sm:grid-cols-2">
                    <div className="flex items-center gap-3">
                      <User className="h-5 w-5 text-muted-foreground" />
                      <div>
                        <p className="text-xs text-muted-foreground">
                          {t('pendingValidation.submittedBy')}
                        </p>
                        <p className="font-medium">
                          {selectedDetail.agentName ?? t('pendingValidation.unknownAgent')}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <Clock className="h-5 w-5 text-muted-foreground" />
                      <div>
                        <p className="text-xs text-muted-foreground">
                          {t('pendingValidation.submittedAt')}
                        </p>
                        <p className="font-medium">{formatDate(selectedDetail.submittedAt)}</p>
                      </div>
                    </div>
                    {selectedDetail.location && (
                      <div className="flex items-center gap-3 sm:col-span-2">
                        <MapPin className="h-5 w-5 text-muted-foreground" />
                        <div>
                          <p className="text-xs text-muted-foreground">
                            {t('pendingValidation.location')}
                          </p>
                          <p className="font-medium">
                            {selectedDetail.location.address ??
                              `${selectedDetail.location.latitude ?? '—'}, ${selectedDetail.location.longitude ?? '—'}`}
                          </p>
                        </div>
                      </div>
                    )}
                  </div>

                  {/* Images */}
                  <div>
                    <h4 className="text-sm font-semibold mb-3">
                      {t('pendingValidation.capturedDocuments')}
                    </h4>
                    <div className="grid gap-4 sm:grid-cols-2">
                      <ImageCard
                        label={t('pendingValidation.frontOfCarteGrise')}
                        url={selectedDetail.frontImageUrl}
                        viewLabel={t('pendingValidation.viewFullSize')}
                      />
                      <ImageCard
                        label={t('pendingValidation.backOfCarteGrise')}
                        url={selectedDetail.backImageUrl}
                        viewLabel={t('pendingValidation.viewFullSize')}
                      />
                    </div>
                  </div>

                  {/* Notes */}
                  {selectedDetail.notes && (
                    <div>
                      <h4 className="text-sm font-semibold mb-2">
                        {t('pendingValidation.agentNotes')}
                      </h4>
                      <p className="text-muted-foreground rounded-lg bg-muted/50 p-3">
                        {selectedDetail.notes}
                      </p>
                    </div>
                  )}

                  {/* Rejection reason (if rejected) */}
                  {selectedDetail.status === 'rejected' && selectedDetail.rejectionReason && (
                    <div className="rounded-lg border border-status-critical/30 bg-status-critical/5 p-4">
                      <h4 className="text-sm font-semibold text-status-critical mb-1">
                        {t('pendingValidation.rejectionReason')}
                      </h4>
                      <p className="text-sm">{selectedDetail.rejectionReason}</p>
                      {selectedDetail.reviewerName && (
                        <p className="text-xs text-muted-foreground mt-2">
                          {t('pendingValidation.rejectedBy', {
                            name: selectedDetail.reviewerName,
                            date: selectedDetail.reviewedAt
                              ? formatDate(selectedDetail.reviewedAt)
                              : '',
                          })}
                        </p>
                      )}
                    </div>
                  )}

                  {/* Approved info (if approved) */}
                  {selectedDetail.status === 'approved' && selectedDetail.reviewerName && (
                    <div className="rounded-lg border border-status-valid/30 bg-status-valid/5 p-4">
                      <h4 className="text-sm font-semibold text-status-valid mb-1">
                        {t('pendingValidation.approvedLabel')}
                      </h4>
                      <p className="text-xs text-muted-foreground">
                        {t('pendingValidation.approvedBy', {
                          name: selectedDetail.reviewerName,
                          date: selectedDetail.reviewedAt
                            ? formatDate(selectedDetail.reviewedAt)
                            : '',
                        })}
                      </p>
                    </div>
                  )}

                  {/* Audit log button */}
                  {selectedDetail.status !== 'pending' && (
                    <Button
                      variant="outline"
                      size="sm"
                      className="gap-2"
                      onClick={() => loadAuditLog(selectedDetail.id)}
                    >
                      <History className="h-4 w-4" />
                      {t('pendingValidation.viewAuditLog')}
                    </Button>
                  )}
                </CardContent>
              </Card>

              {/* ── Audit Log ──────────────────────────────────── */}
              {showAuditLog && auditLog.length > 0 && (
                <Card>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <CardTitle className="flex items-center gap-2 text-base">
                        <History className="h-5 w-5 text-accent" />
                        {t('pendingValidation.auditLog')}
                      </CardTitle>
                      <button
                        onClick={() => setShowAuditLog(false)}
                        className="rounded-lg p-1 hover:bg-muted"
                      >
                        <X className="h-4 w-4" />
                      </button>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-3">
                      {auditLog.map((entry) => (
                        <div
                          key={entry.id}
                          className="flex items-start gap-3 rounded-lg border border-border p-3"
                        >
                          <div
                            className={`mt-0.5 h-8 w-8 rounded-full flex items-center justify-center shrink-0 ${
                              entry.action === 'approved'
                                ? 'bg-status-valid/10 text-status-valid'
                                : 'bg-status-critical/10 text-status-critical'
                            }`}
                          >
                            {entry.action === 'approved' ? (
                              <CheckCircle className="h-4 w-4" />
                            ) : (
                              <XCircle className="h-4 w-4" />
                            )}
                          </div>
                          <div className="flex-1 min-w-0">
                            <p className="text-sm font-medium">
                              {t(`pendingValidation.auditAction_${entry.action}`)}
                            </p>
                            <p className="text-xs text-muted-foreground">
                              {entry.performerName ?? entry.performedBy} •{' '}
                              {formatDate(entry.createdAt)}
                            </p>
                            {entry.reason && (
                              <p className="text-xs text-muted-foreground mt-1 italic">
                                "{entry.reason}"
                              </p>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}
            </>
          ) : (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-16">
                <FileText className="h-16 w-16 text-muted-foreground/30" />
                <p className="mt-4 text-muted-foreground">
                  {t('pendingValidation.selectToReview')}
                </p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </BackOfficeLayout>
  );
}

// ── Sub-components ──────────────────────────────────────────────────────────

function ImageCard({
  label,
  url,
  viewLabel,
}: {
  label: string;
  url: string | null;
  viewLabel: string;
}) {
  const [showFull, setShowFull] = useState(false);

  return (
    <>
      <div className="rounded-lg border border-border overflow-hidden">
        <div className="bg-muted px-3 py-2 text-sm font-medium">{label}</div>
        <div className="aspect-[4/3] bg-muted/50 flex items-center justify-center overflow-hidden">
          {url ? (
            <img
              src={url}
              alt={label}
              className="h-full w-full object-contain"
              onError={(e) => {
                (e.target as HTMLImageElement).style.display = 'none';
                (e.target as HTMLImageElement).parentElement!.innerHTML =
                  '<div class="flex items-center justify-center h-full"><svg class="h-12 w-12 text-muted-foreground/50" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg></div>';
              }}
            />
          ) : (
            <FileText className="h-12 w-12 text-muted-foreground/50" />
          )}
        </div>
        {url && (
          <div className="p-2">
            <Button
              variant="ghost"
              size="sm"
              className="w-full gap-2"
              onClick={() => setShowFull(true)}
            >
              <Eye className="h-4 w-4" />
              {viewLabel}
            </Button>
          </div>
        )}
      </div>

      {/* Full-size image overlay */}
      {showFull && url && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-8"
          onClick={() => setShowFull(false)}
        >
          <div className="relative max-w-4xl max-h-full">
            <button
              className="absolute -top-10 right-0 text-white hover:text-white/80"
              onClick={() => setShowFull(false)}
            >
              <X className="h-6 w-6" />
            </button>
            <img src={url} alt={label} className="max-h-[80vh] rounded-lg shadow-2xl" />
          </div>
        </div>
      )}
    </>
  );
}
