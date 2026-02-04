// Mock Control Logging Service for IVISS
// Tracks all vehicle controls performed by agents

export type ControlStatus = 'valid' | 'warning' | 'critical' | 'pending';

export interface ControlAction {
  type: 'check' | 'flag' | 'citation' | 'impound' | 'release';
  description: string;
  timestamp: Date;
}

export interface ControlRecord {
  id: string;
  plateNumber: string;
  vehicleId?: string;
  agentId: string;
  agentName: string;
  organizationId: string;
  organizationName: string;
  phoneIMEI: string;
  timestamp: Date;
  location: {
    address: string;
    latitude: number;
    longitude: number;
  };
  status: ControlStatus;
  identificationMode: 'manual' | 'photo' | 'live';
  confidence?: number;
  results: {
    registration: ControlStatus;
    insurance: ControlStatus;
    technicalInspection: ControlStatus;
    wantedStatus: ControlStatus;
    customsStatus: ControlStatus;
  };
  actions: ControlAction[];
  notes?: string;
  imageUrl?: string;
}

// Initial mock controls
const mockControls: ControlRecord[] = [
  {
    id: 'ctrl_001',
    plateNumber: 'AB-123-CD',
    vehicleId: 'veh_001',
    agentId: 'usr_agent_001',
    agentName: 'Agent Dupont',
    organizationId: 'org_001',
    organizationName: 'Brigade Alpha - Paris',
    phoneIMEI: '123456789012345',
    timestamp: new Date(Date.now() - 10 * 60 * 1000),
    location: { address: 'Highway A1, KM 42', latitude: 48.8566, longitude: 2.3522 },
    status: 'valid',
    identificationMode: 'manual',
    results: {
      registration: 'valid',
      insurance: 'valid',
      technicalInspection: 'warning',
      wantedStatus: 'valid',
      customsStatus: 'valid',
    },
    actions: [
      {
        type: 'check',
        description: 'Routine control',
        timestamp: new Date(Date.now() - 10 * 60 * 1000),
      },
    ],
  },
  {
    id: 'ctrl_002',
    plateNumber: 'XY-789-ZW',
    vehicleId: 'veh_002',
    agentId: 'usr_agent_002',
    agentName: 'Agent Martin',
    organizationId: 'org_001',
    organizationName: 'Brigade Alpha - Paris',
    phoneIMEI: '234567890123456',
    timestamp: new Date(Date.now() - 25 * 60 * 1000),
    location: { address: 'Rue de Paris, Checkpoint 3', latitude: 48.8606, longitude: 2.3376 },
    status: 'warning',
    identificationMode: 'photo',
    confidence: 94,
    results: {
      registration: 'valid',
      insurance: 'critical',
      technicalInspection: 'valid',
      wantedStatus: 'valid',
      customsStatus: 'valid',
    },
    actions: [
      {
        type: 'check',
        description: 'Random control',
        timestamp: new Date(Date.now() - 25 * 60 * 1000),
      },
      {
        type: 'citation',
        description: 'Citation issued for expired insurance',
        timestamp: new Date(Date.now() - 20 * 60 * 1000),
      },
    ],
    notes:
      'Driver issued citation for expired insurance. Vehicle allowed to proceed to nearest garage.',
  },
  {
    id: 'ctrl_003',
    plateNumber: 'EF-456-GH',
    vehicleId: 'veh_003',
    agentId: 'usr_agent_003',
    agentName: 'Agent Bernard',
    organizationId: 'org_002',
    organizationName: 'Customs - Border',
    phoneIMEI: '345678901234567',
    timestamp: new Date(Date.now() - 60 * 60 * 1000),
    location: { address: 'Border Checkpoint', latitude: 50.6292, longitude: 3.0573 },
    status: 'critical',
    identificationMode: 'live',
    confidence: 97,
    results: {
      registration: 'valid',
      insurance: 'valid',
      technicalInspection: 'valid',
      wantedStatus: 'critical',
      customsStatus: 'valid',
    },
    actions: [
      {
        type: 'check',
        description: 'Live scan detection',
        timestamp: new Date(Date.now() - 60 * 60 * 1000),
      },
      {
        type: 'flag',
        description: 'Vehicle flagged as STOLEN',
        timestamp: new Date(Date.now() - 58 * 60 * 1000),
      },
      {
        type: 'impound',
        description: 'Vehicle impounded. Driver detained.',
        timestamp: new Date(Date.now() - 55 * 60 * 1000),
      },
    ],
    notes: 'STOLEN VEHICLE - Reported stolen on 15/01/2024. Driver detained for questioning.',
  },
  {
    id: 'ctrl_004',
    plateNumber: 'LT-345-AB',
    vehicleId: 'veh_004',
    agentId: 'usr_agent_001',
    agentName: 'Agent Dupont',
    organizationId: 'org_001',
    organizationName: 'Brigade Alpha - Paris',
    phoneIMEI: '123456789012345',
    timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000),
    location: { address: 'Avenue Victor Hugo', latitude: 48.8648, longitude: 2.2908 },
    status: 'valid',
    identificationMode: 'manual',
    results: {
      registration: 'valid',
      insurance: 'valid',
      technicalInspection: 'valid',
      wantedStatus: 'valid',
      customsStatus: 'warning',
    },
    actions: [
      {
        type: 'check',
        description: 'Document verification',
        timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000),
      },
    ],
    notes: 'Customs documents pending review. Driver advised to contact customs office.',
  },
  {
    id: 'ctrl_005',
    plateNumber: 'MN-567-OP',
    vehicleId: 'veh_005',
    agentId: 'usr_agent_004',
    agentName: 'Agent Leroy',
    organizationId: 'org_003',
    organizationName: 'Transport Authority - Lyon',
    phoneIMEI: '456789012345678',
    timestamp: new Date(Date.now() - 4 * 60 * 60 * 1000),
    location: { address: 'Place Bellecour, Lyon', latitude: 45.7578, longitude: 4.832 },
    status: 'valid',
    identificationMode: 'photo',
    confidence: 91,
    results: {
      registration: 'valid',
      insurance: 'valid',
      technicalInspection: 'valid',
      wantedStatus: 'valid',
      customsStatus: 'valid',
    },
    actions: [
      {
        type: 'check',
        description: 'Routine traffic control',
        timestamp: new Date(Date.now() - 4 * 60 * 60 * 1000),
      },
    ],
  },
];

// Statistics
export interface ControlStats {
  today: number;
  thisWeek: number;
  thisMonth: number;
  alerts: number;
  violations: number;
  // Aliases for dashboard
  todayControls: number;
  activeAlerts: number;
  totalVehicles: number;
}

export const mockControlService = {
  // Log a new control
  async logControl(data: {
    plateNumber: string;
    vehicleId?: string;
    agentId: string;
    agentName: string;
    organizationId: string;
    organizationName: string;
    phoneIMEI: string;
    location: { address: string; latitude: number; longitude: number };
    identificationMode: 'manual' | 'photo' | 'live';
    confidence?: number;
    results: ControlRecord['results'];
    notes?: string;
  }): Promise<ControlRecord> {
    await new Promise((resolve) => setTimeout(resolve, 500));

    // Determine overall status
    const statuses = Object.values(data.results);
    let status: ControlStatus = 'valid';
    if (statuses.includes('critical')) status = 'critical';
    else if (statuses.includes('warning')) status = 'warning';

    const control: ControlRecord = {
      id: 'ctrl_' + Math.random().toString(36).substring(2, 8),
      ...data,
      timestamp: new Date(),
      status,
      actions: [{ type: 'check', description: 'Control performed', timestamp: new Date() }],
    };

    mockControls.unshift(control);
    return control;
  },

  // Get controls for agent
  async getControlsByAgent(agentId: string, limit?: number): Promise<ControlRecord[]> {
    await new Promise((resolve) => setTimeout(resolve, 300));
    const controls = mockControls.filter((c) => c.agentId === agentId);
    return limit ? controls.slice(0, limit) : controls;
  },

  // Get today's controls for agent
  async getTodayControlsByAgent(agentId: string): Promise<ControlRecord[]> {
    await new Promise((resolve) => setTimeout(resolve, 300));
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    return mockControls.filter((c) => c.agentId === agentId && c.timestamp >= today);
  },

  // Get all controls (for back office)
  async getAllControls(filters?: {
    startDate?: Date;
    endDate?: Date;
    agentId?: string;
    organizationId?: string;
    status?: ControlStatus;
    plateNumber?: string;
  }): Promise<ControlRecord[]> {
    await new Promise((resolve) => setTimeout(resolve, 400));

    let filtered = [...mockControls];

    if (filters) {
      if (filters.startDate) {
        filtered = filtered.filter((c) => c.timestamp >= filters.startDate!);
      }
      if (filters.endDate) {
        filtered = filtered.filter((c) => c.timestamp <= filters.endDate!);
      }
      if (filters.agentId) {
        filtered = filtered.filter((c) => c.agentId === filters.agentId);
      }
      if (filters.organizationId) {
        filtered = filtered.filter((c) => c.organizationId === filters.organizationId);
      }
      if (filters.status) {
        filtered = filtered.filter((c) => c.status === filters.status);
      }
      if (filters.plateNumber) {
        const plate = filters.plateNumber.toUpperCase().replace(/[-\s]/g, '');
        filtered = filtered.filter((c) =>
          c.plateNumber.replace(/-/g, '').toUpperCase().includes(plate)
        );
      }
    }

    return filtered;
  },

  // Get control by ID
  async getControlById(id: string): Promise<ControlRecord | null> {
    await new Promise((resolve) => setTimeout(resolve, 200));
    return mockControls.find((c) => c.id === id) || null;
  },

  // Add action to control
  async addAction(controlId: string, action: ControlAction): Promise<boolean> {
    await new Promise((resolve) => setTimeout(resolve, 300));

    const control = mockControls.find((c) => c.id === controlId);
    if (control) {
      control.actions.push(action);
      return true;
    }
    return false;
  },

  // Get statistics
  async getStats(organizationId?: string): Promise<ControlStats> {
    await new Promise((resolve) => setTimeout(resolve, 300));

    const now = new Date();
    const startOfDay = new Date(now);
    startOfDay.setHours(0, 0, 0, 0);

    const startOfWeek = new Date(now);
    startOfWeek.setDate(now.getDate() - now.getDay());
    startOfWeek.setHours(0, 0, 0, 0);

    const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);

    const controls = organizationId
      ? mockControls.filter((c) => c.organizationId === organizationId)
      : mockControls;

    return {
      today: controls.filter((c) => c.timestamp >= startOfDay).length,
      thisWeek: controls.filter((c) => c.timestamp >= startOfWeek).length,
      thisMonth: controls.filter((c) => c.timestamp >= startOfMonth).length,
      alerts: controls.filter((c) => c.status === 'critical').length,
      violations: controls.filter((c) => c.status === 'warning' || c.status === 'critical').length,
      // Dashboard aliases
      todayControls: controls.filter((c) => c.timestamp >= startOfDay).length,
      activeAlerts: controls.filter((c) => c.status === 'critical').length,
      totalVehicles: controls.length,
    };
  },

  // Get recent alerts
  async getRecentAlerts(limit: number = 10): Promise<ControlRecord[]> {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return mockControls
      .filter((c) => c.status === 'critical' || c.status === 'warning')
      .slice(0, limit);
  },
};
