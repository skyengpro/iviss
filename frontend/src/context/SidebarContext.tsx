import { createContext, useContext, useState, ReactNode } from 'react';

interface SidebarContextType {
  collapsed: boolean;
  toggle: () => void;
  sidebarWidth: string;
}

export const SidebarContext = createContext<SidebarContextType>({
  collapsed: false,
  toggle: () => {},
  sidebarWidth: '16rem',
});

export function SidebarProvider({ children }: { children: ReactNode }) {
  const [collapsed, setCollapsed] = useState(false);
  return (
    <SidebarContext.Provider
      value={{
        collapsed,
        toggle: () => setCollapsed((p) => !p),
        sidebarWidth: collapsed ? '4.5rem' : '16rem',
      }}
    >
      {children}
    </SidebarContext.Provider>
  );
}
