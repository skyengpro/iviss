import { useState, useEffect } from 'react';
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
} from 'lucide-react';
import { mockVehicleService, PendingVehicle } from '@/services/mockVehicles';
import { toast } from '@/hooks/use-toast';

export default function PendingVehicles() {
  const [pendingVehicles, setPendingVehicles] = useState<PendingVehicle[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedVehicle, setSelectedVehicle] = useState<PendingVehicle | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  useEffect(() => {
    loadPendingVehicles();
  }, []);

  const loadPendingVehicles = async () => {
    try {
      const data = await mockVehicleService.getPendingVehicles();
      setPendingVehicles(data);
    } catch (error) {
      console.error('Failed to load pending vehicles:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleReview = async (id: string, decision: 'approved' | 'rejected') => {
    setIsProcessing(true);

    try {
      await mockVehicleService.reviewPendingVehicle(id, decision);

      toast({
        title: decision === 'approved' ? 'Vehicle Approved' : 'Vehicle Rejected',
        description: `The vehicle has been ${decision}.`,
      });

      // Remove from list
      setPendingVehicles((prev) => prev.filter((v) => v.id !== id));
      setSelectedVehicle(null);
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to process the review.',
        variant: 'destructive',
      });
    } finally {
      setIsProcessing(false);
    }
  };

  const formatDate = (date: Date) => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));

    if (diffHours < 1) return 'Less than an hour ago';
    if (diffHours < 24) return `${diffHours} hours ago`;
    return date.toLocaleDateString();
  };

  return (
    <BackOfficeLayout
      title="Pending Vehicle Validations"
      subtitle={`${pendingVehicles.length} vehicles awaiting review`}
    >
      <div className="grid gap-6 lg:grid-cols-3">
        {/* List */}
        <div className="lg:col-span-1 space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <AlertCircle className="h-5 w-5 text-status-warning" />
                Pending Submissions
              </CardTitle>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <div className="flex items-center justify-center py-8">
                  <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                </div>
              ) : pendingVehicles.length === 0 ? (
                <div className="text-center py-8">
                  <CheckCircle className="mx-auto h-12 w-12 text-status-valid" />
                  <p className="mt-4 text-muted-foreground">No pending validations</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {pendingVehicles.map((vehicle) => (
                    <button
                      key={vehicle.id}
                      onClick={() => setSelectedVehicle(vehicle)}
                      className={`w-full text-left rounded-lg border p-3 transition-colors ${
                        selectedVehicle?.id === vehicle.id
                          ? 'border-accent bg-accent/5'
                          : 'border-border hover:bg-muted'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <span className="font-mono font-bold tracking-wider">
                          {vehicle.plateNumber}
                        </span>
                        <StatusBadge variant="pending" size="sm">
                          Pending
                        </StatusBadge>
                      </div>
                      <div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
                        <User className="h-3 w-3" />
                        <span>{vehicle.submittedBy}</span>
                      </div>
                      <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
                        <Clock className="h-3 w-3" />
                        <span>{formatDate(vehicle.submittedAt)}</span>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Detail View */}
        <div className="lg:col-span-2">
          {selectedVehicle ? (
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="flex items-center gap-2">
                    <FileText className="h-5 w-5 text-accent" />
                    Review Submission
                  </CardTitle>
                  <StatusBadge variant="pending" size="lg">
                    PENDING
                  </StatusBadge>
                </div>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* Vehicle Info */}
                <div className="rounded-lg bg-muted p-4">
                  <p className="text-3xl font-bold font-mono tracking-widest">
                    {selectedVehicle.plateNumber}
                  </p>
                </div>

                {/* Submission Details */}
                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="flex items-center gap-3">
                    <User className="h-5 w-5 text-muted-foreground" />
                    <div>
                      <p className="text-xs text-muted-foreground">Submitted By</p>
                      <p className="font-medium">{selectedVehicle.submittedBy}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <Clock className="h-5 w-5 text-muted-foreground" />
                    <div>
                      <p className="text-xs text-muted-foreground">Submitted At</p>
                      <p className="font-medium">{selectedVehicle.submittedAt.toLocaleString()}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3 sm:col-span-2">
                    <MapPin className="h-5 w-5 text-muted-foreground" />
                    <div>
                      <p className="text-xs text-muted-foreground">Location</p>
                      <p className="font-medium">{selectedVehicle.location}</p>
                    </div>
                  </div>
                </div>

                {/* Documents */}
                <div>
                  <h4 className="text-sm font-semibold mb-3">Captured Documents</h4>
                  <div className="grid gap-4 sm:grid-cols-2">
                    <div className="rounded-lg border border-border overflow-hidden">
                      <div className="bg-muted px-3 py-2 text-sm font-medium">
                        Front of Carte Grise
                      </div>
                      <div className="aspect-[4/3] bg-muted/50 flex items-center justify-center">
                        <FileText className="h-12 w-12 text-muted-foreground/50" />
                      </div>
                      <div className="p-2">
                        <Button variant="ghost" size="sm" className="w-full gap-2">
                          <Eye className="h-4 w-4" />
                          View Full Size
                        </Button>
                      </div>
                    </div>
                    <div className="rounded-lg border border-border overflow-hidden">
                      <div className="bg-muted px-3 py-2 text-sm font-medium">
                        Back of Carte Grise
                      </div>
                      <div className="aspect-[4/3] bg-muted/50 flex items-center justify-center">
                        <FileText className="h-12 w-12 text-muted-foreground/50" />
                      </div>
                      <div className="p-2">
                        <Button variant="ghost" size="sm" className="w-full gap-2">
                          <Eye className="h-4 w-4" />
                          View Full Size
                        </Button>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Notes */}
                {selectedVehicle.notes && (
                  <div>
                    <h4 className="text-sm font-semibold mb-2">Agent Notes</h4>
                    <p className="text-muted-foreground rounded-lg bg-muted/50 p-3">
                      {selectedVehicle.notes}
                    </p>
                  </div>
                )}

                {/* Actions */}
                <div className="flex gap-3 pt-4 border-t border-border">
                  <Button
                    className="flex-1 gap-2 bg-status-valid text-status-valid-foreground hover:bg-status-valid/90"
                    onClick={() => handleReview(selectedVehicle.id, 'approved')}
                    disabled={isProcessing}
                  >
                    <CheckCircle className="h-5 w-5" />
                    Approve & Add to Database
                  </Button>
                  <Button
                    variant="outline"
                    className="flex-1 gap-2 text-status-critical hover:text-status-critical hover:bg-status-critical/10"
                    onClick={() => handleReview(selectedVehicle.id, 'rejected')}
                    disabled={isProcessing}
                  >
                    <XCircle className="h-5 w-5" />
                    Reject
                  </Button>
                </div>
              </CardContent>
            </Card>
          ) : (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-16">
                <FileText className="h-16 w-16 text-muted-foreground/30" />
                <p className="mt-4 text-muted-foreground">Select a pending submission to review</p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </BackOfficeLayout>
  );
}
