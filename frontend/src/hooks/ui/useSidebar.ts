import { useContext } from 'react';
import { SidebarContext } from '@/context/SidebarContext';

function useSidebar() {
  return useContext(SidebarContext);
}

export { useSidebar };
