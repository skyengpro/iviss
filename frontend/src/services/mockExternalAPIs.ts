// Mock External API Service for IVISS
// Simulates calls to partner APIs (Insurance, Police, Customs, Technical Inspection)

export type APIStatus = 'valid' | 'warning' | 'critical' | 'unknown';

export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  responseTime: number;
}

export interface InsuranceResult {
  status: APIStatus;
  provider?: string;
  policyNumber?: string;
  expiryDate?: string;
  coverageType?: string;
  notes?: string;
}

export interface PoliceResult {
  status: APIStatus;
  isWanted: boolean;
  isStolen: boolean;
  reportDate?: string;
  reportNumber?: string;
  notes?: string;
}

export interface CustomsResult {
  status: APIStatus;
  isCleared: boolean;
  importDate?: string;
  declarationNumber?: string;
  notes?: string;
}

export interface TechnicalInspectionResult {
  status: APIStatus;
  lastInspectionDate?: string;
  expiryDate?: string;
  mileage?: number;
  defects?: string[];
  notes?: string;
}

export interface AggregatedVehicleStatus {
  insurance: InsuranceResult;
  police: PoliceResult;
  customs: CustomsResult;
  technicalInspection: TechnicalInspectionResult;
  overallStatus: APIStatus;
  queryTime: number;
}

// Mock data lookup table
const mockAPIData: Record<
  string,
  {
    insurance: InsuranceResult;
    police: PoliceResult;
    customs: CustomsResult;
    technical: TechnicalInspectionResult;
  }
> = {
  AB123CD: {
    insurance: {
      status: 'valid',
      provider: 'AXA France',
      policyNumber: 'AXA-2024-123456',
      expiryDate: 'Jun 2024',
      coverageType: 'Full Coverage',
    },
    police: { status: 'valid', isWanted: false, isStolen: false },
    customs: { status: 'valid', isCleared: true, importDate: 'Jan 2021' },
    technical: {
      status: 'warning',
      lastInspectionDate: 'Mar 2022',
      expiryDate: 'Mar 2024',
      mileage: 45000,
      notes: 'Inspection expires in 2 months',
    },
  },
  XY789ZW: {
    insurance: {
      status: 'critical',
      provider: 'Allianz',
      policyNumber: 'ALZ-2023-789012',
      expiryDate: 'Jan 2024',
      notes: 'POLICY EXPIRED - Vehicle is uninsured',
    },
    police: { status: 'valid', isWanted: false, isStolen: false },
    customs: { status: 'valid', isCleared: true },
    technical: {
      status: 'valid',
      lastInspectionDate: 'Nov 2022',
      expiryDate: 'Nov 2024',
      mileage: 62000,
    },
  },
  EF456GH: {
    insurance: {
      status: 'valid',
      provider: 'MAIF',
      policyNumber: 'MAIF-2023-456789',
      expiryDate: 'Sep 2024',
    },
    police: {
      status: 'critical',
      isWanted: true,
      isStolen: true,
      reportDate: '15/01/2024',
      reportNumber: 'POL-2024-00147',
      notes: 'STOLEN VEHICLE - Report to authorities immediately',
    },
    customs: { status: 'valid', isCleared: true },
    technical: {
      status: 'valid',
      lastInspectionDate: 'Jul 2023',
      expiryDate: 'Jul 2024',
      mileage: 78000,
    },
  },
  LT345AB: {
    insurance: {
      status: 'valid',
      provider: 'Allianz',
      policyNumber: 'ALZ-2024-345678',
      expiryDate: 'Apr 2024',
    },
    police: { status: 'valid', isWanted: false, isStolen: false },
    customs: {
      status: 'warning',
      isCleared: false,
      importDate: 'Dec 2023',
      declarationNumber: 'CUS-2023-98765',
      notes: 'Import documents under review',
    },
    technical: {
      status: 'valid',
      lastInspectionDate: 'Feb 2024',
      expiryDate: 'Feb 2025',
      mileage: 25000,
    },
  },
  MN567OP: {
    insurance: {
      status: 'valid',
      provider: 'GMF',
      policyNumber: 'GMF-2024-567890',
      expiryDate: 'Dec 2024',
      coverageType: 'Third Party',
    },
    police: { status: 'valid', isWanted: false, isStolen: false },
    customs: { status: 'valid', isCleared: true },
    technical: {
      status: 'valid',
      lastInspectionDate: 'Jan 2024',
      expiryDate: 'Jan 2026',
      mileage: 15000,
    },
  },
  QR890ST: {
    insurance: {
      status: 'valid',
      provider: 'MACIF',
      policyNumber: 'MAC-2023-890123',
      expiryDate: 'Aug 2024',
    },
    police: { status: 'valid', isWanted: false, isStolen: false },
    customs: { status: 'valid', isCleared: true },
    technical: {
      status: 'critical',
      lastInspectionDate: 'Nov 2021',
      expiryDate: 'Nov 2023',
      mileage: 95000,
      defects: ['Brake pads worn', 'Tire tread low'],
      notes: 'INSPECTION EXPIRED - Vehicle should not be on road',
    },
  },
};

function normalizePlate(plate: string): string {
  return plate.toUpperCase().replace(/[-\s]/g, '');
}

function randomDelay(min: number, max: number): Promise<void> {
  const delay = min + Math.random() * (max - min);
  return new Promise((resolve) => setTimeout(resolve, delay));
}

export const mockExternalAPIService = {
  // Query Insurance API
  async checkInsurance(plateNumber: string): Promise<APIResponse<InsuranceResult>> {
    const startTime = Date.now();
    await randomDelay(200, 600);

    const normalized = normalizePlate(plateNumber);
    const data = mockAPIData[normalized];

    if (data) {
      return {
        success: true,
        data: data.insurance,
        responseTime: Date.now() - startTime,
      };
    }

    return {
      success: true,
      data: { status: 'unknown', notes: 'No insurance record found' },
      responseTime: Date.now() - startTime,
    };
  },

  // Query Police/Wanted Vehicles API
  async checkPolice(plateNumber: string): Promise<APIResponse<PoliceResult>> {
    const startTime = Date.now();
    await randomDelay(300, 800);

    const normalized = normalizePlate(plateNumber);
    const data = mockAPIData[normalized];

    if (data) {
      return {
        success: true,
        data: data.police,
        responseTime: Date.now() - startTime,
      };
    }

    return {
      success: true,
      data: { status: 'valid', isWanted: false, isStolen: false },
      responseTime: Date.now() - startTime,
    };
  },

  // Query Customs API
  async checkCustoms(plateNumber: string): Promise<APIResponse<CustomsResult>> {
    const startTime = Date.now();
    await randomDelay(250, 700);

    const normalized = normalizePlate(plateNumber);
    const data = mockAPIData[normalized];

    if (data) {
      return {
        success: true,
        data: data.customs,
        responseTime: Date.now() - startTime,
      };
    }

    return {
      success: true,
      data: { status: 'valid', isCleared: true },
      responseTime: Date.now() - startTime,
    };
  },

  // Query Technical Inspection API
  async checkTechnicalInspection(
    plateNumber: string
  ): Promise<APIResponse<TechnicalInspectionResult>> {
    const startTime = Date.now();
    await randomDelay(200, 500);

    const normalized = normalizePlate(plateNumber);
    const data = mockAPIData[normalized];

    if (data) {
      return {
        success: true,
        data: data.technical,
        responseTime: Date.now() - startTime,
      };
    }

    return {
      success: true,
      data: { status: 'unknown', notes: 'No inspection record found' },
      responseTime: Date.now() - startTime,
    };
  },

  // Aggregate all API calls
  async checkAllSystems(plateNumber: string): Promise<AggregatedVehicleStatus> {
    const startTime = Date.now();

    // Run all API calls in parallel
    const [insuranceRes, policeRes, customsRes, technicalRes] = await Promise.all([
      this.checkInsurance(plateNumber),
      this.checkPolice(plateNumber),
      this.checkCustoms(plateNumber),
      this.checkTechnicalInspection(plateNumber),
    ]);

    const insurance = insuranceRes.data || { status: 'unknown' as APIStatus };
    const police = policeRes.data || {
      status: 'unknown' as APIStatus,
      isWanted: false,
      isStolen: false,
    };
    const customs = customsRes.data || { status: 'unknown' as APIStatus, isCleared: true };
    const technicalInspection = technicalRes.data || { status: 'unknown' as APIStatus };

    // Determine overall status
    let overallStatus: APIStatus = 'valid';
    const statuses = [insurance.status, police.status, customs.status, technicalInspection.status];

    if (statuses.includes('critical')) {
      overallStatus = 'critical';
    } else if (statuses.includes('warning')) {
      overallStatus = 'warning';
    } else if (statuses.includes('unknown')) {
      overallStatus = 'warning';
    }

    return {
      insurance,
      police,
      customs,
      technicalInspection,
      overallStatus,
      queryTime: Date.now() - startTime,
    };
  },
};
