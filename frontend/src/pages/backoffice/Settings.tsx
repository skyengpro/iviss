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
import { toast } from 'sonner';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { getAccessToken } from '@/services/auth/tokenManager';
import {
  User,
  Lock,
  Bell,
  Building2,
  Eye,
  EyeOff,
  LogOut,
  CheckCircle2,
  Globe,
  Moon,
  Sun,
  Monitor,
} from 'lucide-react';

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

function PasswordSection() {
  const { t } = useTranslation();
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
      toast.error(t('settings.security.passwordMismatchError'));
      return;
    }
    setIsLoading(true);
    try {
      const token = getAccessToken();
      const baseUrl = import.meta.env.VITE_API_URL || '';
      const res = await fetch(`${baseUrl}/api/v1/auth/change-password`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
        body: JSON.stringify({ newPassword, confirmPassword }),
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        toast.error(data.message || t('settings.security.updateError'));
      } else {
        toast.success(t('settings.security.updateSuccess'));
        setNewPassword('');
        setConfirmPassword('');
      }
    } catch {
      toast.error(t('settings.security.updateError'));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="new-pw">{t('settings.security.newPassword')}</Label>
        <div className="relative">
          <Input
            id="new-pw"
            type={showNew ? 'text' : 'password'}
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            placeholder={t('settings.security.newPasswordPlaceholder')}
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
        <Label htmlFor="confirm-pw">{t('settings.security.confirmPassword')}</Label>
        <div className="relative">
          <Input
            id="confirm-pw"
            type={showConfirm ? 'text' : 'password'}
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            placeholder={t('settings.security.confirmPasswordPlaceholder')}
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
        <p className="text-xs text-destructive">{t('settings.security.passwordTooShort')}</p>
      )}
      {newPassword.length >= 8 && confirmPassword && newPassword !== confirmPassword && (
        <p className="text-xs text-destructive">{t('settings.security.passwordMismatch')}</p>
      )}
      {newPassword.length >= 8 && confirmPassword && newPassword === confirmPassword && (
        <p className="flex items-center gap-1 text-xs text-emerald-600">
          <CheckCircle2 className="h-3.5 w-3.5" /> {t('settings.security.passwordMatch')}
        </p>
      )}

      <Button type="submit" disabled={!canSubmit} className="w-full sm:w-auto">
        {isLoading ? t('settings.security.updating') : t('settings.security.updateButton')}
      </Button>
    </form>
  );
}

export default function Settings() {
  const { t, i18n } = useTranslation();
  const { user, logout } = useAuth();
  const navigate = useNavigate();

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
    <BackOfficeLayout title={t('settings.title')} subtitle={t('settings.subtitle')}>
      <div className="mx-auto max-w-4xl space-y-6">
        {/* Profile banner */}
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
                {t('settings.signOut')}
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Tabs */}
        <Tabs defaultValue="profile">
          <TabsList className="grid w-full grid-cols-3 sm:w-auto sm:inline-flex">
            <TabsTrigger value="profile" className="gap-2">
              <User className="h-4 w-4" />
              <span className="hidden sm:inline">{t('settings.tabs.profile')}</span>
            </TabsTrigger>
            <TabsTrigger value="security" className="gap-2">
              <Lock className="h-4 w-4" />
              <span className="hidden sm:inline">{t('settings.tabs.security')}</span>
            </TabsTrigger>
            <TabsTrigger value="preferences" className="gap-2">
              <Bell className="h-4 w-4" />
              <span className="hidden sm:inline">{t('settings.tabs.preferences')}</span>
            </TabsTrigger>
          </TabsList>

          {/* Profile tab */}
          <TabsContent value="profile" className="mt-4 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <User className="h-4 w-4" />
                  {t('settings.profile.title')}
                </CardTitle>
                <CardDescription>{t('settings.profile.description')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-1">
                <SectionTitle>{t('settings.profile.identity')}</SectionTitle>
                <InfoRow label={t('settings.profile.fullName')} value={user?.name} />
                <Separator />
                <InfoRow label={t('settings.profile.username')} value={user?.username} />
                <Separator />
                <InfoRow label={t('settings.profile.email')} value={user?.email} />
                <Separator />
                <InfoRow label={t('settings.profile.phone')} value={user?.phoneNumber} />
                {user?.badgeId && (
                  <>
                    <Separator />
                    <InfoRow label={t('settings.profile.badgeId')} value={user.badgeId} />
                  </>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          {/* Security tab */}
          <TabsContent value="security" className="mt-4 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Lock className="h-4 w-4" />
                  {t('settings.security.changePasswordTitle')}
                </CardTitle>
                <CardDescription>
                  {t('settings.security.changePasswordDescription')}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <PasswordSection />
              </CardContent>
            </Card>

            <Card className="border-destructive/20">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base text-destructive">
                  <LogOut className="h-4 w-4" />
                  {t('settings.security.signOutTitle')}
                </CardTitle>
                <CardDescription>{t('settings.security.signOutDescription')}</CardDescription>
              </CardHeader>
              <CardContent>
                <Button variant="destructive" onClick={handleLogout} className="gap-2">
                  <LogOut className="h-4 w-4" />
                  {t('settings.security.signOutButton')}
                </Button>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Preferences tab */}
          <TabsContent value="preferences" className="mt-4 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Monitor className="h-4 w-4" />
                  {t('settings.preferences.appearanceTitle')}
                </CardTitle>
                <CardDescription>{t('settings.preferences.appearanceDescription')}</CardDescription>
              </CardHeader>
              <CardContent>
                <SectionTitle>{t('settings.preferences.theme')}</SectionTitle>
                <div className="flex gap-2">
                  {[
                    { value: 'light', icon: Sun, label: t('settings.preferences.themeLight') },
                    { value: 'dark', icon: Moon, label: t('settings.preferences.themeDark') },
                    {
                      value: 'system',
                      icon: Monitor,
                      label: t('settings.preferences.themeSystem'),
                    },
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
                  {t('settings.preferences.themeComingSoon')}
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Globe className="h-4 w-4" />
                  {t('settings.preferences.languageTitle')}
                </CardTitle>
                <CardDescription>{t('settings.preferences.languageDescription')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-1">
                <InfoRow
                  label={t('settings.preferences.language')}
                  value={i18n.language === 'fr' ? 'Français (fr)' : 'English (en)'}
                />
                <Separator />
                <InfoRow
                  label={t('settings.preferences.timezone')}
                  value="UTC+1 (West Africa Time)"
                />
                <Separator />
                <InfoRow label={t('settings.preferences.dateFormat')} value="DD/MM/YYYY" />
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </BackOfficeLayout>
  );
}
