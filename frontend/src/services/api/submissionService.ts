import { fetchWithAuth } from './backendFetch';

// ── Types ─────────────────────────────────────────────────────────────────────

export type SubmissionStatus = 'pending' | 'approved' | 'rejected';

export type SubmissionSource = 'submission' | 's3_unregistered';

export interface PendingSubmissionListItem {
  /** `null` for an S3 `unregistered/` entry — no detail to fetch. */
  id: string | null;
  plateNumber: string;
  agentName: string | null;
  status: SubmissionStatus;
  submittedAt: string;
  source: SubmissionSource;
}

export interface SubmissionLocation {
  latitude: number | null;
  longitude: number | null;
  address: string | null;
}

export interface VehicleDataEntry {
  chassisNumber: string;
  brand: string;
  model: string;
  year: number;
  color?: string;
  enginePower?: string;
  fuelType?: string;
  ownerName: string;
  ownerAddress?: string;
  ownerNationalId?: string;
}

export interface PendingSubmissionDetail {
  id: string;
  plateNumber: string;
  agentId: string;
  agentName: string | null;
  location: SubmissionLocation | null;
  frontImageUrl: string | null;
  backImageUrl: string | null;
  notes: string | null;
  status: SubmissionStatus;
  submittedAt: string;
  reviewedAt: string | null;
  reviewedBy: string | null;
  reviewerName: string | null;
  rejectionReason: string | null;
  vehicleData: VehicleDataEntry | null;
}

export interface SubmissionAuditLogEntry {
  id: string;
  action: string;
  performedBy: string;
  performerName: string | null;
  reason: string | null;
  details: unknown;
  createdAt: string;
}

// ── API Functions ─────────────────────────────────────────────────────────────

export async function getSubmissions(
  status?: SubmissionStatus
): Promise<PendingSubmissionListItem[]> {
  const params = status ? `?status=${status}` : '';
  const response = await fetchWithAuth(`/api/v1/admin/submissions${params}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch submissions: ${response.status}`);
  }
  return response.json();
}

export async function getSubmissionById(id: string): Promise<PendingSubmissionDetail> {
  const response = await fetchWithAuth(`/api/v1/admin/submissions/${id}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch submission: ${response.status}`);
  }
  return response.json();
}

export async function getSubmissionAuditLog(id: string): Promise<SubmissionAuditLogEntry[]> {
  const response = await fetchWithAuth(`/api/v1/admin/submissions/${id}/audit`);
  if (!response.ok) {
    throw new Error(`Failed to fetch audit log: ${response.status}`);
  }
  return response.json();
}
