import { createContext } from 'react';

export interface SidebarContextType {
  collapsed: boolean;
  toggle: () => void;
  sidebarWidth: string;
}

export const SidebarContext = createContext<SidebarContextType>({
  collapsed: false,
  toggle: () => {},
  sidebarWidth: '16rem',
});
