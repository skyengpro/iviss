import { useState } from "react";
import { MobileHeader } from "./MobileHeader";
import { MobileNavigation } from "./MobileNavigation";
import { MobileSidebar } from "./MobileSidebar";
import { cn } from "@/lib/utils";

interface MobileLayoutProps {
  children: React.ReactNode;
  title?: string;
  className?: string;
  hideNavigation?: boolean;
}

export function MobileLayout({
  children,
  title,
  className,
  hideNavigation = false,
}: MobileLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="min-h-screen bg-background">
      <MobileHeader title={title} onMenuClick={() => setSidebarOpen(true)} />
      
      <MobileSidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />

      <main
        className={cn(
          "pt-16 pb-20",
          hideNavigation && "pb-4",
          className
        )}
      >
        {children}
      </main>

      {!hideNavigation && <MobileNavigation />}
    </div>
  );
}
