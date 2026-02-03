// Mock Authentication Service for IVISS
// Simulates backend authentication with predefined users

export type UserRole = 'agent' | 'supervisor' | 'admin';

export interface User {
  id: string;
  username: string;
  email: string;
  role: UserRole;
  name: string;
  organization: string;
  organizationId: string;
  badgeId: string;
  phoneIMEI: string;
  avatarInitials: string;
  isActive: boolean;
}

export interface AuthSession {
  user: User;
  token: string;
  expiresAt: Date;
}

// Mock user database
const mockUsers: Record<string, { password: string; user: User }> = {
  agent01: {
    password: 'agent123',
    user: {
      id: 'usr_agent_001',
      username: 'agent01',
      email: 'agent.dupont@police.gov',
      role: 'agent',
      name: 'Agent Dupont',
      organization: 'Brigade Alpha - Paris',
      organizationId: 'org_001',
      badgeId: 'PA-2024-0147',
      phoneIMEI: generateIMEI(),
      avatarInitials: 'AD',
      isActive: true,
    },
  },
  supervisor01: {
    password: 'supervisor123',
    user: {
      id: 'usr_supervisor_001',
      username: 'supervisor01',
      email: 'supervisor.martin@police.gov',
      role: 'supervisor',
      name: 'Supervisor Martin',
      organization: 'Brigade Alpha - Paris',
      organizationId: 'org_001',
      badgeId: 'PS-2024-0023',
      phoneIMEI: generateIMEI(),
      avatarInitials: 'SM',
      isActive: true,
    },
  },
  admin01: {
    password: 'admin123',
    user: {
      id: 'usr_admin_001',
      username: 'admin01',
      email: 'admin@iviss.gov',
      role: 'admin',
      name: 'Admin User',
      organization: 'IVISS Central',
      organizationId: 'org_central',
      badgeId: 'ADM-2024-0001',
      phoneIMEI: generateIMEI(),
      avatarInitials: 'AU',
      isActive: true,
    },
  },
};

function generateIMEI(): string {
  const digits = Array.from({ length: 15 }, () => Math.floor(Math.random() * 10));
  return digits.join('');
}

function generateToken(): string {
  return 'tok_' + Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
}

// Storage keys
const SESSION_KEY = 'iviss_session';

export const mockAuthService = {
  // Authenticate user
  async login(username: string, password: string): Promise<{ success: boolean; session?: AuthSession; error?: string }> {
    // Simulate network delay
    await new Promise((resolve) => setTimeout(resolve, 800));

    const userRecord = mockUsers[username.toLowerCase()];

    if (!userRecord) {
      return { success: false, error: 'User not found. Please check your credentials.' };
    }

    if (userRecord.password !== password) {
      return { success: false, error: 'Incorrect password. Please try again.' };
    }

    if (!userRecord.user.isActive) {
      return { success: false, error: 'This account has been deactivated. Contact your administrator.' };
    }

    const session: AuthSession = {
      user: { ...userRecord.user, phoneIMEI: generateIMEI() },
      token: generateToken(),
      expiresAt: new Date(Date.now() + 8 * 60 * 60 * 1000), // 8 hours
    };

    // Store session
    localStorage.setItem(SESSION_KEY, JSON.stringify(session));

    return { success: true, session };
  },

  // Logout
  async logout(): Promise<void> {
    await new Promise((resolve) => setTimeout(resolve, 200));
    localStorage.removeItem(SESSION_KEY);
  },

  // Get current session
  getSession(): AuthSession | null {
    const sessionData = localStorage.getItem(SESSION_KEY);
    if (!sessionData) return null;

    try {
      const session: AuthSession = JSON.parse(sessionData);
      session.expiresAt = new Date(session.expiresAt);

      // Check if expired
      if (session.expiresAt < new Date()) {
        localStorage.removeItem(SESSION_KEY);
        return null;
      }

      return session;
    } catch {
      localStorage.removeItem(SESSION_KEY);
      return null;
    }
  },

  // Check if authenticated
  isAuthenticated(): boolean {
    return this.getSession() !== null;
  },

  // Get current user
  getCurrentUser(): User | null {
    const session = this.getSession();
    return session?.user || null;
  },

  // Get mock credentials for display
  getMockCredentials() {
    return [
      { role: 'Agent', username: 'agent01', password: 'agent123' },
      { role: 'Supervisor', username: 'supervisor01', password: 'supervisor123' },
      { role: 'Admin', username: 'admin01', password: 'admin123' },
    ];
  },
};
