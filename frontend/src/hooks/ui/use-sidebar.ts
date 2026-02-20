import * as React from 'react';
import { useTranslation } from 'react-i18next';

export type SidebarContext = {
  state: 'expanded' | 'collapsed';
  open: boolean;
  setOpen: (open: boolean) => void;
  openMobile: boolean;
  setOpenMobile: (open: boolean) => void;
  isMobile: boolean;
  toggleSidebar: () => void;
};

export const SidebarContext = React.createContext<SidebarContext | null>(null);

export function useSidebar() {
  const { t } = useTranslation();
  const context = React.useContext(SidebarContext);
  if (!context) {
    throw new Error(t('errors.useSidebar'));
  }

  return context;
}
