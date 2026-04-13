import { useContext } from 'react';
import { SidebarContext } from '@/context/SidebarContext/SidebarContext';

function useSidebar() {
  return useContext(SidebarContext);
}

export { useSidebar };
