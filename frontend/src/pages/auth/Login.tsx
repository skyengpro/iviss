import { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Shield, Eye, EyeOff, Smartphone, Monitor, Info } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useAuth } from '@/contexts/AuthContext';

type AppType = 'mobile' | 'backoffice';

export default function Login() {
  const navigate = useNavigate();
  const location = useLocation();
  const { login, isAuthenticated, user, getMockCredentials } = useAuth();

  const [appType, setAppType] = useState<AppType>('mobile');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [showCredentials, setShowCredentials] = useState(false);

  const mockCredentials = getMockCredentials();

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated && user) {
      const from = (location.state as { from?: string } | null)?.from;
      if (from) {
        navigate(from);
      } else if (user.role === 'admin') {
        navigate('/backoffice');
      } else {
        navigate('/mobile');
      }
    }
  }, [isAuthenticated, user, navigate, location.state]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    const result = await login(username, password);

    if (result.success) {
      // Navigation handled by useEffect
    } else {
      setError(result.error || 'Login failed');
    }

    setIsLoading(false);
  };

  const fillCredentials = (cred: { username: string; password: string }) => {
    setUsername(cred.username);
    setPassword(cred.password);
    setError('');
  };

  return (
    <div className="min-h-[100dvh] bg-gradient-to-br from-navy-900 via-navy-800 to-navy-900 flex flex-col items-center justify-center p-4 overflow-y-auto scrollbar-none">
      {/* Background pattern */}
      <div className="absolute inset-0 opacity-5">
        <div
          className="absolute inset-0"
          style={{
            backgroundImage: `url("data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23ffffff' fill-opacity='1'%3E%3Cpath d='M36 34v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6 34v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6 4V0H4v4H0v2h4v4h2V6h4V4H6z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E")`,
          }}
        />
      </div>

      <div className="w-full max-w-md relative z-10 animate-fade-in py-6">
        {/* Logo */}
        <div className="mb-6 text-center">
          <div className="inline-flex h-14 w-14 items-center justify-center rounded-2xl bg-accent shadow-lg">
            <Shield className="h-8 w-8 text-accent-foreground" />
          </div>
          <h1 className="mt-3 text-2xl font-bold text-white tracking-wide">IVISS</h1>
          <p className="mt-1 text-xs text-white/60 px-4">
            Intelligent Vehicle Identification & Status System
          </p>
        </div>

        <Card className="border-0 shadow-2xl glass card-elevated">
          <CardHeader className="space-y-1 pb-4">
            <CardTitle className="text-2xl text-center">Sign In</CardTitle>
            <CardDescription className="text-center">
              Access your {appType === 'mobile' ? 'Mobile Agent' : 'Back Office'} account
            </CardDescription>
          </CardHeader>

          <CardContent>
            {/* App type selector */}
            <div className="mb-6 flex rounded-lg bg-muted p-1">
              <button
                type="button"
                onClick={() => setAppType('mobile')}
                className={cn(
                  'flex flex-1 items-center justify-center gap-2 rounded-md px-4 py-2.5 text-sm font-medium transition-all',
                  appType === 'mobile'
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                )}
              >
                <Smartphone className="h-4 w-4" />
                Mobile Agent
              </button>
              <button
                type="button"
                onClick={() => setAppType('backoffice')}
                className={cn(
                  'flex flex-1 items-center justify-center gap-2 rounded-md px-4 py-2.5 text-sm font-medium transition-all',
                  appType === 'backoffice'
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                )}
              >
                <Monitor className="h-4 w-4" />
                Back Office
              </button>
            </div>

            {/* Demo credentials helper */}
            <div className="mb-4">
              <button
                type="button"
                onClick={() => setShowCredentials(!showCredentials)}
                className="flex w-full items-center justify-between rounded-lg bg-accent/5 border border-accent/20 px-4 py-2 text-sm text-accent hover:bg-accent/10 transition-colors"
              >
                <span className="flex items-center gap-2">
                  <Info className="h-4 w-4" />
                  Demo Credentials
                </span>
                <span className="text-xs opacity-70">{showCredentials ? 'Hide' : 'Show'}</span>
              </button>

              {showCredentials && (
                <div className="mt-2 space-y-2 rounded-lg border border-border bg-muted/50 p-3">
                  {mockCredentials.map(
                    (cred: { username: string; password: string; role: string }) => (
                      <button
                        key={cred.username}
                        type="button"
                        onClick={() => fillCredentials(cred)}
                        className="flex w-full items-center justify-between rounded-md bg-background px-3 py-2 text-left text-sm hover:bg-accent/10 transition-colors"
                      >
                        <div>
                          <span className="font-medium">{cred.role}</span>
                          <span className="ml-2 text-muted-foreground">
                            {cred.username} / {cred.password}
                          </span>
                        </div>
                        <span className="text-xs text-accent">Use</span>
                      </button>
                    )
                  )}
                </div>
              )}
            </div>

            <form onSubmit={handleSubmit} className="space-y-4">
              {error && (
                <div className="rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
                  {error}
                </div>
              )}

              <div className="space-y-2">
                <Label htmlFor="username">Username</Label>
                <Input
                  id="username"
                  type="text"
                  placeholder="agent01"
                  value={username}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => setUsername(e.target.value)}
                  required
                  className="h-11"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <div className="relative">
                  <Input
                    id="password"
                    type={showPassword ? 'text' : 'password'}
                    placeholder="••••••••"
                    value={password}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                      setPassword(e.target.value)
                    }
                    required
                    className="h-11 pr-10"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  >
                    {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  </button>
                </div>
              </div>

              <Button
                type="submit"
                className="w-full h-11 bg-accent text-accent-foreground hover:bg-accent/90"
                disabled={isLoading}
              >
                {isLoading ? (
                  <div className="h-5 w-5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                ) : (
                  'Sign In'
                )}
              </Button>
            </form>

            <div className="mt-4 text-center">
              <a href="#" className="text-sm text-muted-foreground hover:text-accent">
                Forgot your password?
              </a>
            </div>

            {/* Security notice */}
            <div className="mt-4 rounded-lg bg-muted/30 p-2.5 text-center text-[10px] leading-tight text-muted-foreground/80">
              <Shield className="inline h-3 w-3 mr-1 opacity-70" />
              Secured government system. Unauthorized access prohibited.
            </div>
          </CardContent>
        </Card>

        {/* Footer */}
        <p className="mt-4 text-center text-[10px] text-white/30">
          © 2024 IVISS - National Vehicle Control System
        </p>
      </div>
    </div>
  );
}
