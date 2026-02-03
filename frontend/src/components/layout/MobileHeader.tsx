import { Menu, Bell, User, Shield } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface MobileHeaderProps {
  onMenuClick: () => void;
  title?: string;
  className?: string;
}

export function MobileHeader({ onMenuClick, title = "IVISS", className }: MobileHeaderProps) {
  return (
    <header
      className={cn(
        "fixed left-0 right-0 top-0 z-50 header-gradient",
        className
      )}
    >
      <div className="flex h-16 items-center justify-between px-4">
        {/* Menu button */}
        <Button
          variant="ghost"
          size="icon"
          onClick={onMenuClick}
          className="touch-target text-primary-foreground hover:bg-white/10"
        >
          <Menu className="h-6 w-6" />
        </Button>

        {/* Logo and title */}
        <div className="flex items-center gap-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent">
            <Shield className="h-5 w-5 text-accent-foreground" />
          </div>
          <span className="text-lg font-bold text-primary-foreground tracking-wide">
            {title}
          </span>
        </div>

        {/* Right actions */}
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="touch-target relative text-primary-foreground hover:bg-white/10"
          >
            <Bell className="h-5 w-5" />
            <span className="absolute right-2 top-2 h-2 w-2 rounded-full bg-status-critical" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="touch-target text-primary-foreground hover:bg-white/10"
          >
            <User className="h-5 w-5" />
          </Button>
        </div>
      </div>
    </header>
  );
}
