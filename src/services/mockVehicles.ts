// Mock Vehicle Database Service for IVISS
// Simulates national vehicle registry with various statuses

export type VehicleStatus = 'valid' | 'warning' | 'critical' | 'pending';

export interface VehicleOwner {
  name: string;
  address: string;
  nationalId?: string;
}

export interface VehicleDocument {
  status: VehicleStatus;
  expiryDate?: string;
  provider?: string;
  notes?: string;
}

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

// Mock vehicle database
const mockVehicles: Vehicle[] = [
  {
    id: 'veh_001',
    plateNumber: 'AB-123-CD',
    chassisNumber: 'VF1RFB00651234567',
    brand: 'Renault',
    model: 'Clio',
    year: 2021,
    color: 'Blue',
    enginePower: '100 HP',
    fuelType: 'Petrol',
    owner: {
      name: 'Jean Dupont',
      address: '15 Rue de la Paix, 75001 Paris',
      nationalId: 'FR-1234567890',
    },
    registration: { status: 'valid', expiryDate: 'Dec 2025' },
    insurance: { status: 'valid', provider: 'AXA France', expiryDate: 'Jun 2024' },
    technicalInspection: { status: 'warning', expiryDate: 'Mar 2024', notes: 'Expires in 2 months' },
    wantedStatus: { status: 'valid' },
    customsStatus: { status: 'valid' },
  },
  {
    id: 'veh_002',
    plateNumber: 'XY-789-ZW',
    chassisNumber: 'WVWZZZ3CZWE123456',
    brand: 'Volkswagen',
    model: 'Golf',
    year: 2019,
    color: 'Black',
    enginePower: '150 HP',
    fuelType: 'Diesel',
    owner: {
      name: 'Marie Lambert',
      address: '42 Avenue des Champs, 69001 Lyon',
      nationalId: 'FR-9876543210',
    },
    registration: { status: 'valid', expiryDate: 'Aug 2025' },
    insurance: { status: 'critical', expiryDate: 'Jan 2024', notes: 'EXPIRED - Vehicle uninsured' },
    technicalInspection: { status: 'valid', expiryDate: 'Nov 2024' },
    wantedStatus: { status: 'valid' },
    customsStatus: { status: 'valid' },
  },
  {
    id: 'veh_003',
    plateNumber: 'EF-456-GH',
    chassisNumber: 'WAUZZZ8V1GA123456',
    brand: 'Audi',
    model: 'A4',
    year: 2016,
    color: 'Silver',
    enginePower: '190 HP',
    fuelType: 'Diesel',
    owner: {
      name: 'Pierre Martin',
      address: '8 Boulevard Victor Hugo, 33000 Bordeaux',
      nationalId: 'FR-5555555555',
    },
    registration: { status: 'valid', expiryDate: 'May 2025' },
    insurance: { status: 'valid', provider: 'MAIF', expiryDate: 'Sep 2024' },
    technicalInspection: { status: 'valid', expiryDate: 'Jul 2024' },
    wantedStatus: { status: 'critical', reason: 'STOLEN - Reported 15/01/2024' },
    customsStatus: { status: 'valid' },
  },
  {
    id: 'veh_004',
    plateNumber: 'LT-345-AB',
    chassisNumber: 'WBAPH5C50BA123456',
    brand: 'BMW',
    model: '320d',
    year: 2020,
    color: 'White',
    enginePower: '190 HP',
    fuelType: 'Diesel',
    owner: {
      name: 'Sophie Bernard',
      address: '25 Rue Nationale, 59000 Lille',
      nationalId: 'FR-7777777777',
    },
    registration: { status: 'valid', expiryDate: 'Oct 2025' },
    insurance: { status: 'valid', provider: 'Allianz', expiryDate: 'Apr 2024' },
    technicalInspection: { status: 'valid', expiryDate: 'Feb 2025' },
    wantedStatus: { status: 'valid' },
    customsStatus: { status: 'warning', notes: 'Import documents pending review' },
  },
  {
    id: 'veh_005',
    plateNumber: 'MN-567-OP',
    chassisNumber: 'TMBJF25L5B1234567',
    brand: 'Peugeot',
    model: '308',
    year: 2022,
    color: 'Red',
    enginePower: '130 HP',
    fuelType: 'Petrol',
    owner: {
      name: 'Lucas Moreau',
      address: '12 Place du Marché, 13001 Marseille',
      nationalId: 'FR-8888888888',
    },
    registration: { status: 'valid', expiryDate: 'Mar 2026' },
    insurance: { status: 'valid', provider: 'GMF', expiryDate: 'Dec 2024' },
    technicalInspection: { status: 'valid', expiryDate: 'Jan 2026' },
    wantedStatus: { status: 'valid' },
    customsStatus: { status: 'valid' },
  },
  {
    id: 'veh_006',
    plateNumber: 'QR-890-ST',
    chassisNumber: 'VF7FCNFUC89123456',
    brand: 'Citroën',
    model: 'C4',
    year: 2018,
    color: 'Grey',
    enginePower: '110 HP',
    fuelType: 'Diesel',
    owner: {
      name: 'Emma Rousseau',
      address: '7 Rue de la Liberté, 44000 Nantes',
      nationalId: 'FR-6666666666',
    },
    registration: { status: 'warning', expiryDate: 'Feb 2024', notes: 'Expires soon' },
    insurance: { status: 'valid', provider: 'MACIF', expiryDate: 'Aug 2024' },
    technicalInspection: { status: 'critical', expiryDate: 'Nov 2023', notes: 'EXPIRED' },
    wantedStatus: { status: 'valid' },
    customsStatus: { status: 'valid' },
  },
];

// Pending vehicle submissions (from agents in the field)
export interface PendingVehicle {
  id: string;
  plateNumber: string;
  submittedBy: string;
  submittedAt: Date;
  location: string;
  frontImageUrl: string;
  backImageUrl: string;
  status: 'pending' | 'approved' | 'rejected';
  notes?: string;
}

const pendingVehicles: PendingVehicle[] = [
  {
    id: 'pend_001',
    plateNumber: 'ZZ-999-AA',
    submittedBy: 'Agent Dupont',
    submittedAt: new Date(Date.now() - 2 * 60 * 60 * 1000),
    location: 'Highway A1, KM 42',
    frontImageUrl: '/placeholder.svg',
    backImageUrl: '/placeholder.svg',
    status: 'pending',
    notes: 'Vehicle with foreign plates, driver claims recent import',
  },
  {
    id: 'pend_002',
    plateNumber: 'XX-111-BB',
    submittedBy: 'Agent Martin',
    submittedAt: new Date(Date.now() - 5 * 60 * 60 * 1000),
    location: 'Border Checkpoint Lille',
    frontImageUrl: '/placeholder.svg',
    backImageUrl: '/placeholder.svg',
    status: 'pending',
  },
];

export const mockVehicleService = {
  // Search vehicle by plate number
  async searchByPlate(plateNumber: string): Promise<{ found: boolean; vehicle?: Vehicle }> {
    await new Promise((resolve) => setTimeout(resolve, 1200));

    const normalizedPlate = plateNumber.toUpperCase().replace(/\s/g, '');
    const vehicle = mockVehicles.find(
      (v) => v.plateNumber.replace(/-/g, '').toUpperCase() === normalizedPlate.replace(/-/g, '')
    );

    if (vehicle) {
      return { found: true, vehicle };
    }

    return { found: false };
  },

  // Get vehicle by ID
  async getById(id: string): Promise<Vehicle | null> {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return mockVehicles.find((v) => v.id === id) || null;
  },

  // Get all vehicles (for back office)
  async getAll(): Promise<Vehicle[]> {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return [...mockVehicles];
  },

  // Get vehicles with issues
  async getFlaggedVehicles(): Promise<Vehicle[]> {
    await new Promise((resolve) => setTimeout(resolve, 400));
    return mockVehicles.filter(
      (v) =>
        v.wantedStatus.status === 'critical' ||
        v.insurance.status === 'critical' ||
        v.technicalInspection.status === 'critical'
    );
  },

  // Submit pending vehicle (when not found)
  async submitPendingVehicle(data: {
    plateNumber: string;
    submittedBy: string;
    location: string;
    frontImage: string;
    backImage: string;
    notes?: string;
  }): Promise<PendingVehicle> {
    await new Promise((resolve) => setTimeout(resolve, 800));

    const pending: PendingVehicle = {
      id: 'pend_' + Math.random().toString(36).substring(2, 8),
      plateNumber: data.plateNumber,
      submittedBy: data.submittedBy,
      submittedAt: new Date(),
      location: data.location,
      frontImageUrl: data.frontImage,
      backImageUrl: data.backImage,
      status: 'pending',
      notes: data.notes,
    };

    pendingVehicles.push(pending);
    return pending;
  },

  // Get pending vehicles (for admin)
  async getPendingVehicles(): Promise<PendingVehicle[]> {
    await new Promise((resolve) => setTimeout(resolve, 400));
    return pendingVehicles.filter((v) => v.status === 'pending');
  },

  // Approve/reject pending vehicle
  async reviewPendingVehicle(
    id: string,
    decision: 'approved' | 'rejected',
    notes?: string
  ): Promise<boolean> {
    await new Promise((resolve) => setTimeout(resolve, 600));

    const pending = pendingVehicles.find((v) => v.id === id);
    if (pending) {
      pending.status = decision;
      if (notes) pending.notes = notes;
      return true;
    }
    return false;
  },

  // Simulate live scan detection
  async simulateLiveScan(): Promise<{ plateNumber: string; confidence: number; status: VehicleStatus }> {
    await new Promise((resolve) => setTimeout(resolve, 2000 + Math.random() * 2000));

    const randomVehicle = mockVehicles[Math.floor(Math.random() * mockVehicles.length)];
    const status: VehicleStatus =
      randomVehicle.wantedStatus.status === 'critical'
        ? 'critical'
        : randomVehicle.insurance.status === 'critical' || randomVehicle.technicalInspection.status === 'critical'
        ? 'warning'
        : 'valid';

    return {
      plateNumber: randomVehicle.plateNumber,
      confidence: 85 + Math.floor(Math.random() * 14),
      status,
    };
  },
};
