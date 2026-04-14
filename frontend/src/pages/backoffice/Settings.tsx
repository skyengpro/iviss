import { useState } from 'react';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { useAuth } from '@/hooks/auth/use-auth';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import { toast } from 'sonner';
import { useNavigate } from 'react-router-dom';
import { getAccessToken } from '@/services/auth/tokenManager';
import {
  User,
  Lock,
  Bell,
  Shield,
  Building2,
  Eye,
  EyeOff,
  LogOut,
  CheckCircle2,
  Info,
  Globe,
  Moon,
  Sun,
  Monitor,
} from 'lucide-react';

// ── small helper ──────────────────────────────────────────────────────────────
function InfoRow({ label, value }: { label: string; value?: string | null }) {
  return (
    <div className="flex items-center justify-between py-2.5">
      <span className="text-sm text-muted-foreground">{label}</span>
      <span className="text-sm font-medium">{value || '—'}</span>
    </div>
  );
}

function SectionTitle({ children }: { children: React.ReactNode }) {
  return (
    <p className="mb-3 text-xs font-semibold uppercase tracking-widest text-muted-foreground">
      {children}
    </p>
  );
}

// ── Role badge ────────────────────────────────────────────────────────────────
function RoleBadge({ role }: { role: string }) {
  const map: Record<string, { label: string; className: string }> = {
    admin: {
      label: 'Super Admin',
      className: 'bg-destructive/10 text-destructive border-destructive/20',
    },
    org_admin: { label: 'Org Admin', className: 'bg-primary/10 text-primary border-primary/20' },
    manager: {
      label: 'Supervisor',
      className: 'bg-amber-500/10 text-amber-600 border-amber-500/20',
    },
    agent: { label: 'Agent', className: 'bg-muted text-muted-foreground' },
  };
  const { label, className } = map[role] ?? {
    label: role,
    className: 'bg-muted text-muted-foreground',
  };
  return (
    <Badge variant="outline" className={className}>
      {label}
    </Badge>
  );
}

// ── Password change section ───────────────────────────────────────────────────
function PasswordSection() {
  const navigate = useNavigate();
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showNew, setShowNew] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const canSubmit = newPassword.trim().length >= 8 && confirmPassword.trim() && !isLoading;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (newPassword !== confirmPassword) {
      toast.error('Passwords do not match');
      return;
    }
    setIsLoading(true);
    try {
      const token = getAccessToken();
      const baseUrl = import.meta.env.VITE_API_URL || 'http://localhost:3000';
      const res = await fetch(`${baseUrl}/api/v1/auth/change-password`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
        body: JSON.stringify({ newPassword, confirmPassword }),
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        toast.error(data.message || 'Failed to update password');
      } else {
        toast.success('Password updated successfully');
        setNewPassword('');
        setConfirmPassword('');
      }
    } catch {
      toast.error('Failed to update password');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="new-pw">New Password</Label>
        <div className="relative">
          <Input
            id="new-pw"
            type={showNew ? 'text' : 'password'}
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            placeholder="At least 8 characters"
            className="pr-10"
          />
          <button
            type="button"
            onClick={() => setShowNew((v) => !v)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            tabIndex={-1}
          >
            {showNew ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        </div>
      </div>

      <div className="space-y-2">
        <Label htmlFor="confirm-pw">Confirm New Password</Label>
        <div className="relative">
          <Input
            id="confirm-pw"
            type={showConfirm ? 'text' : 'password'}
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            placeholder="Re-enter new password"
            className="pr-10"
          />
          <button
            type="button"
            onClick={() => setShowConfirm((v) => !v)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            tabIndex={-1}
          >
            {showConfirm ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        </div>
      </div>

      {newPassword && newPassword.length < 8 && (
        <p className="text-xs text-destructive">Password must be at least 8 characters</p>
      )}
      {newPassword.length >= 8 && confirmPassword && newPassword !== confirmPassword && (
        <p className="text-xs text-destructive">Passwords do not match</p>
      )}
      {newPassword.length >= 8 && confirmPassword && newPassword === confirmPassword && (
        <p className="flex items-center gap-1 text-xs text-emerald-600">
          <CheckCircle2 className="h-3.5 w-3.5" /> Passwords match
        </p>
      )}

      <Button type="submit" disabled={!canSubmit} className="w-full sm:w-auto">
        {isLoading ? 'Updating…' : 'Update Password'}
      </Button>
    </form>
  );
}

// ── Main page ─────────────────────────────────────────────────────────────────
export default function Settings() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const isSuperAdmin = user?.role === 'admin';
  const isOrgAdmin = user?.role === 'org_admin';

  // Notification prefs (local state — extend with API when ready)
  const [notifSessions, setNotifSessions] = useState(true);
  const [notifAlerts, setNotifAlerts] = useState(true);
  const [notifReports, setNotifReports] = useState(false);

  const handleLogout = async () => {
    await logout();
    navigate('/admin-login');
  };

  const initials = user?.name
    ? user.name
        .split(' ')
        .map((n) => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2)
    : 'AD';

  return (
    <BackOfficeLayout title="Settings" subtitle="Manage your account and system preferences">
      <div className="mx-auto max-w-4xl space-y-6">
        {/* ── Profile banner ── */}
        <Card>
          <CardContent className="pt-6">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:gap-6">
              <Avatar className="h-16 w-16 shrink-0">
                <AvatarFallback className="bg-primary text-primary-foreground text-xl font-bold">
                  {initials}
                </AvatarFallback>
              </Avatar>
              <div className="flex-1 min-w-0">
                <div className="flex flex-wrap items-center gap-2">
                  <h2 className="text-xl font-semibold truncate">
                    {user?.name || 'Administrator'}
                  </h2>
                  {user?.role && <RoleBadge role={user.role} />}
                </div>
                <p className="mt-0.5 text-sm text-muted-foreground truncate">
                  {user?.email || user?.username || '—'}
                </p>
                {user?.organization && (
                  <p className="mt-0.5 flex items-center gap-1.5 text-sm text-muted-foreground">
                    <Building2 className="h-3.5 w-3.5 shrink-0" />
                    {user.organization}
                  </p>
                )}
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={handleLogout}
                className="shrink-0 gap-2 text-destructive hover:bg-destructive/5 hover:text-destructive border-destructive/30"
              >
                <LogOut className="h-4 w-4" />
                Sign Out
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* ── Tabs ── */}
        <Tabs defaultValue="profile">
          <TabsList className="grid w-full grid-cols-3 sm:w-auto sm:inline-flex">
            <TabsTrigger value="profile" className="gap-2">
              <User className="h-4 w-4" />
              <span className="hidden sm:inline">Profile</span>
            </TabsTrigger>
            <TabsTrigger value="security" className="gap-2">
              <Lock className="h-4 w-4" />
              <span className="hidden sm:inline">Security</span>
            </TabsTrigger>
            <TabsTrigger value="preferences" className="gap-2">
              <Bell className="h-4 w-4" />
              <span className="hidden sm:inline">Preferences</span>
            </TabsTrigger>
          </TabsList>

          {/* ── Profile tab ── */}
          <TabsContent value="profile" className="mt-4 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <User className="h-4 w-4" />
                  Account Information
                </CardTitle>
                <CardDescription>Your identity and access details</CardDescription>
              </CardHeader>
              <CardContent className="space-y-1">
                <SectionTitle>Identity</SectionTitle>
                <InfoRow label="Full Name" value={user?.name} />
                <Separator />
                <InfoRow label="Username" value={user?.username} />
                <Separator />
                <InfoRow label="Email" value={user?.email} />
                <Separator />
                <InfoRow label="Phone" value={user?.phoneNumber} />
                {user?.badgeId && (
                  <>
                    <Separator />
                    <InfoRow label="Badge ID" value={user.badgeId} />
                  </>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Shield className="h-4 w-4" />
                  Access & Role
                </CardTitle>
                <CardDescription>Your permissions within the system</CardDescription>
              </CardHeader>
              <CardContent className="space-y-1">
                <SectionTitle>Permissions</SectionTitle>
                <InfoRow
                  label="Role"
                  value={
                    user?.role === 'admin'
                      ? 'Super Administrator'
                      : user?.role === 'org_admin'
                        ? 'Organization Administrator'
                        : user?.role === 'manager'
                          ? 'Supervisor'
                          : 'Agent'
                  }
                />
                <Separator />
                <InfoRow
                  label="Account Status"
                  value={user?.status === 'ACTIVE' ? 'Active' : user?.status}
                />
                {user?.organization && (
                  <>
                    <Separator />
                    <InfoRow label="Organization" value={user.organization} />
                  </>
                )}
                <Separator />
                <div className="flex items-center justify-between py-2.5">
                  <span className="text-sm text-muted-foreground">Permissions</span>
                  <div className="flex flex-wrap justify-end gap-1.5">
                    {isSuperAdmin && (
                      <>
                        <Badge variant="secondary" className="text-xs">
                          All Organizations
                        </Badge>
                        <Badge variant="secondary" className="text-xs">
                          Create Org Admins
                        </Badge>
                        <Badge variant="secondary" className="text-xs">
                          Full Access
                        </Badge>
                      </>
                    )}
                    {isOrgAdmin && (
                      <>
                        <Badge variant="secondary" className="text-xs">
                          Org Users
                        </Badge>
                        <Badge variant="secondary" className="text-xs">
                          Create Agents
                        </Badge>
                        <Badge variant="secondary" className="text-xs">
                          Org Reports
                        </Badge>
                      </>
                    )}
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* System info — superadmin only */}
            {isSuperAdmin && (
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Globe className="h-4 w-4" />
                    System Information
                  </CardTitle>
                  <CardDescription>Platform and environment details</CardDescription>
                </CardHeader>
                <CardContent className="space-y-1">
                  <SectionTitle>Platform</SectionTitle>
                  <InfoRow
                    label="System"
                    value="IVISS — Intelligent Vehicle Identification & Status System"
                  />
                  <Separator />
                  <InfoRow label="Version" value="v1.0.0" />
                  <Separator />
                  <InfoRow label="Environment" value="Production" />
                  <Separator />
                  <InfoRow
                    label="API"
                    value={import.meta.env.VITE_API_URL || 'http://localhost:3000'}
                  />
                </CardContent>
              </Card>
            )}
          </TabsContent>

          {/* ── Security tab ── */}
          <TabsContent value="security" className="mt-4 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Lock className="h-4 w-4" />
                  Change Password
                </CardTitle>
                <CardDescription>
                  Update your password. Use at least 8 characters with a mix of letters and numbers.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <PasswordSection />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Info className="h-4 w-4" />
                  Session Information
                </CardTitle>
                <CardDescription>Details about your current session</CardDescription>
              </CardHeader>
              <CardContent className="space-y-1">
                <SectionTitle>Current Session</SectionTitle>
                <InfoRow label="Session Type" value="Web (JWT)" />
                <Separator />
                <InfoRow
                  label="Access Level"
                  value={
                    isSuperAdmin
                      ? 'Global — all organizations'
                      : isOrgAdmin
                        ? `Scoped — ${user?.organization || 'your organization'}`
                        : 'Standard'
                  }
                />
                <Separator />
                <InfoRow label="Token Lifetime" value="24 hours (auto-refresh)" />
                <Separator />
                <div className="flex items-center justify-between py-2.5">
                  <span className="text-sm text-muted-foreground">Session Status</span>
                  <Badge
                    className="bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                    variant="outline"
                  >
                    <span className="mr-1.5 inline-block h-1.5 w-1.5 rounded-full bg-emerald-500" />
                    Active
                  </Badge>
                </div>
              </CardContent>
            </Card>

            <Card className="border-destructive/20">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base text-destructive">
                  <LogOut className="h-4 w-4" />
                  Sign Out
                </CardTitle>
                <CardDescription>
                  End your current session. You will need to sign in again to access the
                  back-office.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Button variant="destructive" onClick={handleLogout} className="gap-2">
                  <LogOut className="h-4 w-4" />
                  Sign Out of IVISS
                </Button>
              </CardContent>
            </Card>
          </TabsContent>

          {/* ── Preferences tab ── */}
          <TabsContent value="preferences" className="mt-4 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Bell className="h-4 w-4" />
                  Notifications
                </CardTitle>
                <CardDescription>Choose what events you want to be notified about</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <SectionTitle>In-App Alerts</SectionTitle>

                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium">Session Events</p>
                    <p className="text-xs text-muted-foreground">
                      Alerts when agent sessions are terminated or restarted
                    </p>
                  </div>
                  <Switch checked={notifSessions} onCheckedChange={setNotifSessions} />
                </div>
                <Separator />

                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium">Critical Alerts</p>
                    <p className="text-xs text-muted-foreground">
                      Flagged vehicles and high-priority control events
                    </p>
                  </div>
                  <Switch checked={notifAlerts} onCheckedChange={setNotifAlerts} />
                </div>
                <Separator />

                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium">Report Ready</p>
                    <p className="text-xs text-muted-foreground">
                      Notify when a generated report is available
                    </p>
                  </div>
                  <Switch checked={notifReports} onCheckedChange={setNotifReports} />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Monitor className="h-4 w-4" />
                  Appearance
                </CardTitle>
                <CardDescription>Customize how the interface looks</CardDescription>
              </CardHeader>
              <CardContent>
                <SectionTitle>Theme</SectionTitle>
                <div className="flex gap-2">
                  {[
                    { value: 'light', icon: Sun, label: 'Light' },
                    { value: 'dark', icon: Moon, label: 'Dark' },
                    { value: 'system', icon: Monitor, label: 'System' },
                  ].map(({ value, icon: Icon, label }) => (
                    <button
                      key={value}
                      className="flex flex-1 flex-col items-center gap-1.5 rounded-lg border border-border bg-muted/40 px-3 py-3 text-xs font-medium text-muted-foreground transition hover:border-primary/40 hover:bg-muted hover:text-foreground"
                    >
                      <Icon className="h-4 w-4" />
                      {label}
                    </button>
                  ))}
                </div>
                <p className="mt-2 text-xs text-muted-foreground">
                  Theme switching coming soon — currently follows your system preference.
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Globe className="h-4 w-4" />
                  Language & Region
                </CardTitle>
                <CardDescription>Interface language and regional settings</CardDescription>
              </CardHeader>
              <CardContent className="space-y-1">
                <InfoRow label="Language" value="English (en)" />
                <Separator />
                <InfoRow label="Timezone" value="UTC+1 (West Africa Time)" />
                <Separator />
                <InfoRow label="Date Format" value="DD/MM/YYYY" />
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </BackOfficeLayout>
  );
}
