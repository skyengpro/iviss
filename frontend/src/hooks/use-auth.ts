import { createContext, useContext } from 'react';
import { User, AuthSession } from '@/services/mockAuth';

export interface AuthContextType {
  readonly user: User | null;
  readonly session: AuthSession | null;
  readonly isLoading: boolean;
  readonly isAuthenticated: boolean;
  login: (username: string, password: string) => Promise<{ success: boolean; error?: string }>;
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
