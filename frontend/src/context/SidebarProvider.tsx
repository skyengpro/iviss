import { useState, ReactNode } from 'react';
import { SidebarContext } from './SidebarContext';

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
