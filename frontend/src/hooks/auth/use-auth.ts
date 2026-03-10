import { createContext, useContext } from 'react';
import { UserProfile, AuthResponse } from '@/openapi-rq/requests/types.gen';

export interface AuthContextType {
  user: UserProfile | null;
  session: AuthResponse | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  login: (username: string, password: string) => Promise<{ success: boolean; error?: string }>;
  activate: (params: {
    badgeId: string;
    activationCode: string;
    deviceId: string;
    publicKeyBase64: string;
  }) => Promise<{ success: boolean; error?: string }>;
  logout: () => Promise<void>;
  getMockCredentials: () => { role: string; username: string; password: string }[];
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
