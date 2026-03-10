import { BackOfficeSidebar } from './BackOfficeSidebar';
import { BackOfficeHeader } from './BackOfficeHeader';
import { SidebarProvider, useSidebar } from '@/context/SidebarContext';
import { cn } from '@/lib/utils';

interface BackOfficeLayoutProps {
  children: React.ReactNode;
  title?: string;
  subtitle?: string;
  actions?: React.ReactNode;
  className?: string;
}

function BackOfficeLayoutInner({
  children,
  title,
  subtitle,
  actions,
  className,
}: BackOfficeLayoutProps) {
  const { sidebarWidth } = useSidebar();
  return (
    <div className="min-h-screen bg-background">
      <BackOfficeSidebar />

      <div
        className="transition-all duration-300 ease-in-out"
        style={{ paddingLeft: sidebarWidth }}
      >
        <BackOfficeHeader title={title} subtitle={subtitle} actions={actions} />
        <main className={cn('p-6', className)}>{children}</main>
      </div>
    </div>
  );
}

export function BackOfficeLayout(props: BackOfficeLayoutProps) {
  return (
    <SidebarProvider>
      <BackOfficeLayoutInner {...props} />
    </SidebarProvider>
  );
}
