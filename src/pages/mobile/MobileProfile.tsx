import { MobileLayout } from "@/components/layout/MobileLayout";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/ui/status-badge";
import { 
  User, 
  Shield, 
  Smartphone, 
  Building2, 
  LogOut,
  ChevronRight,
  HelpCircle,
  FileText,
  Settings
} from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { useNavigate } from "react-router-dom";

export default function MobileProfile() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  if (!user) return null;

  return (
    <MobileLayout title="Profile">
      <div className="p-4 space-y-6">
        {/* Profile Header */}
        <div className="rounded-xl bg-gradient-to-br from-primary to-primary/80 p-6 text-primary-foreground">
          <div className="flex items-center gap-4">
            <div className="flex h-16 w-16 items-center justify-center rounded-full bg-white/20 text-2xl font-bold">
              {user.avatarInitials}
            </div>
            <div>
              <h2 className="text-xl font-bold">{user.name}</h2>
              <p className="text-sm opacity-80">{user.role.toUpperCase()}</p>
              <StatusBadge variant="valid" size="sm" className="mt-1">
                Active
              </StatusBadge>
            </div>
          </div>
        </div>

        {/* User Info */}
        <section className="rounded-xl border border-border bg-card overflow-hidden">
          <div className="p-4 border-b border-border">
            <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
              Account Information
            </h3>
          </div>
          
          <InfoRow icon={User} label="Badge ID" value={user.badgeId} />
          <InfoRow icon={Shield} label="Role" value={user.role} />
          <InfoRow icon={Building2} label="Organization" value={user.organization} />
          <InfoRow icon={Smartphone} label="Phone IMEI" value={user.phoneIMEI.slice(0, 8) + '...'} />
        </section>

        {/* Quick Links */}
        <section className="rounded-xl border border-border bg-card overflow-hidden">
          <div className="p-4 border-b border-border">
            <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
              Quick Actions
            </h3>
          </div>
          
          <MenuLink icon={FileText} label="My Controls Today" badge="12" />
          <MenuLink icon={HelpCircle} label="Help & Support" />
          <MenuLink icon={Settings} label="Settings" />
        </section>

        {/* Logout */}
        <Button
          variant="outline"
          className="w-full h-12 gap-2 text-destructive hover:text-destructive hover:bg-destructive/10"
          onClick={handleLogout}
        >
          <LogOut className="h-5 w-5" />
          Sign Out
        </Button>

        {/* App Info */}
        <div className="text-center text-xs text-muted-foreground">
          <p>IVISS Mobile v1.0.0</p>
          <p className="mt-1">© 2024 National Vehicle Control System</p>
        </div>
      </div>
    </MobileLayout>
  );
}

function InfoRow({ 
  icon: Icon, 
  label, 
  value 
}: { 
  icon: React.ElementType; 
  label: string; 
  value: string; 
}) {
  return (
    <div className="flex items-center gap-3 px-4 py-3 border-b border-border last:border-b-0">
      <Icon className="h-5 w-5 text-muted-foreground" />
      <div className="flex-1 min-w-0">
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="font-medium truncate">{value}</p>
      </div>
    </div>
  );
}

function MenuLink({ 
  icon: Icon, 
  label, 
  badge 
}: { 
  icon: React.ElementType; 
  label: string; 
  badge?: string;
}) {
  return (
    <button className="flex w-full items-center gap-3 px-4 py-3 border-b border-border last:border-b-0 hover:bg-muted transition-colors">
      <Icon className="h-5 w-5 text-muted-foreground" />
      <span className="flex-1 text-left">{label}</span>
      {badge && (
        <span className="rounded-full bg-accent px-2 py-0.5 text-xs font-medium text-accent-foreground">
          {badge}
        </span>
      )}
      <ChevronRight className="h-4 w-4 text-muted-foreground" />
    </button>
  );
}
