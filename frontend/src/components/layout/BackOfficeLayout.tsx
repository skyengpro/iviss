import { BackOfficeSidebar } from './BackOfficeSidebar';
import { BackOfficeHeader } from './BackOfficeHeader';
import { cn } from '@/lib/utils';

interface BackOfficeLayoutProps {
  readonly children: React.ReactNode;
  readonly title?: string;
  readonly subtitle?: string;
  readonly actions?: React.ReactNode;
  readonly className?: string;
}

export function BackOfficeLayout({
  children,
  title,
  subtitle,
  actions,
  className,
}: BackOfficeLayoutProps) {
  return (
    <div className="min-h-screen bg-background">
      <BackOfficeSidebar />

      <div className="pl-64">
        <BackOfficeHeader title={title} subtitle={subtitle} actions={actions} />

        <main className={cn('p-6', className)}>{children}</main>
      </div>
    </div>
  );
}
