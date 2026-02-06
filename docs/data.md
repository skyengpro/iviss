# Data Schema & Constants

This document details the data structures and constants used throughout the IVISS application.

## 1. Core Data Models

### 1.1 Vehicle
The central entity representing a vehicle in the registry.

```typescript
export interface Vehicle {
  id: string;
  plateNumber: string;
  chassisNumber: string;
  brand: string;
  model: string;
  year: number;
  color: string;
  enginePower: string;
  fuelType: string;
  owner: VehicleOwner;
  registration: VehicleDocument;
  insurance: VehicleDocument;
  technicalInspection: VehicleDocument;
  wantedStatus: VehicleDocument & { reason?: string };
  customsStatus: VehicleDocument;
}

export interface VehicleOwner {
  name: string;
  address: string;
  nationalId?: string;
}

export interface InsuranceStatus {
  status: VehicleStatus;
  provider?: string;
  policyNumber?: string;
  expiryDate?: string;
  coverageType?: string;
  notes?: string;
}

export interface PoliceStatus {
  status: VehicleStatus;
  isWanted: boolean;
  isStolen: boolean;
  reportDate?: string;
  reportNumber?: string;
  notes?: string;
}

export interface CustomsStatus {
  status: VehicleStatus;
  isCleared: boolean;
  importDate?: string;
  declarationNumber?: string;
  notes?: string;
}

export interface TechnicalStatus {
  status: VehicleStatus;
  lastInspectionDate?: string;
  expiryDate?: string;
  mileage?: number;
  defects?: string[];
  notes?: string;
}
```

### 1.2 Control Record
A log of a vehicle check performed by an agent.

```typescript
export interface ControlRecord {
  id: string;
  plateNumber: string;
  vehicleId?: string;
  agentId: string;
  agentName: string;
  organizationId: string;
  organizationName: string;
  phoneIMEI: string;
  timestamp: string; // Changed from Date to string
  location: {
    address: string;
    longitude: number;
  };
  status: VehicleStatus;
  identificationMode: 'manual' | 'photo' | 'live';
  confidence?: number;
  results: {
    registration: VehicleStatus;
    insurance: VehicleStatus;
    technicalInspection: VehicleStatus;
    wantedStatus: VehicleStatus;
    customsStatus: VehicleStatus;
  };
  actions: ControlAction[];
  notes?: string;
  imageUrl?: string;
}

export interface ControlAction {
  type: 'check' | 'flag' | 'citation' | 'impound' | 'release';
  description: string;
  timestamp: Date;
}
```

### 1.3 User
The authenticated user (agent or back-office staff).

```typescript
export interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'agent' | 'manager';
  organizationId: string;
  organization: string;
  badgeId: string;
  phoneIMEI: string;
  avatarInitials: string;
  isActive: boolean;
}
```

## 2. Enums and Constants

### 2.1 Vehicle Status
Used to indicate the compliance level of various vehicle requirements.

| Value | Description |
| :--- | :--- |
| `valid` | Fully compliant, no issues. |
| `warning` | Issue detected but not critical (e.g., expiring soon). |
| `critical` | Major violation or dangerous state (e.g., stolen, no insurance). |
| `pending` | Awaiting validation or processing. |

### 2.2 Identification Modes
How the vehicle was identified during the control.

| Value | Description |
| :--- | :--- |
| `manual` | Agent typed the plate number. |
| `photo` | OCR performed on a static image. |
| `live` | OCR performed on a live video stream. |

## 3. API Query Parameters for Filters

When fetching control history or dashboard data, the following parameters are used:

- `start_date`: ISO 8601 string (e.g., `2024-01-01T00:00:00Z`).
- `end_date`: ISO 8601 string.
- `agent_id`: Unique identifier for the agent.
- `status`: One of `valid`, `warning`, `critical`, `pending`.
- `plate`: Substring or exact plate number for filtering.
