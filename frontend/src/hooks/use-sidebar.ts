import * as React from 'react';
import { useTranslation } from 'react-i18next';

export type SidebarContext = {
  readonly state: 'expanded' | 'collapsed';
  readonly open: boolean;
  readonly setOpen: (open: boolean) => void;
  readonly openMobile: boolean;
  readonly setOpenMobile: (open: boolean) => void;
  readonly isMobile: boolean;
  readonly toggleSidebar: () => void;
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
