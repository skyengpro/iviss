import { useEffect, useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ShieldCheck, LogIn } from 'lucide-react';
import { useAuth } from '@/hooks/auth/use-auth';

export default function AdminLogin() {
  const navigate = useNavigate();
  const { login, isAuthenticated, user } = useAuth();

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const canSubmit = !!email.trim() && !!password.trim() && !isLoading;

  // Redirect if already logged in
  useEffect(() => {
    if (isAuthenticated && user) {
      if (user.role === 'admin' || user.role === 'manager') {
        navigate('/backoffice');
      } else {
        navigate('/mobile');
      }
    }
  }, [isAuthenticated, user, navigate]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!email.trim()) {
      setError('Email is required');
      return;
    }

    if (!password.trim()) {
      setError('Password is required');
      return;
    }

    setIsLoading(true);
    try {
      const result = await login(email.trim(), password);

      if (!result.success) {
        setError(result.error || 'Login failed');
      }
      // Navigation handled by the useEffect above when isAuthenticated changes
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-primary/5 via-background to-background">
      <div className="mx-auto flex min-h-screen w-full max-w-md flex-col justify-center px-4 py-10">
        <div className="mb-6 flex flex-col items-center text-center">
          <div className="mb-3 flex h-12 w-12 items-center justify-center rounded-2xl bg-primary text-primary-foreground shadow-sm">
            <ShieldCheck className="h-6 w-6" />
          </div>
          <h1 className="text-2xl font-semibold tracking-tight">IVISS</h1>
          <p className="mt-1 text-sm text-muted-foreground">Back-Office Administration</p>
        </div>

        <Card className="w-full">
          <CardHeader>
            <CardTitle className="text-xl">Admin Sign In</CardTitle>
            <CardDescription>
              Enter your administrator credentials to access the back-office.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="admin-email">Email</Label>
                <Input
                  id="admin-email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="admin@iviss.gov"
                  autoComplete="email"
                  autoFocus
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="admin-password">Password</Label>
                <Input
                  id="admin-password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  autoComplete="current-password"
                />
              </div>

              {error ? <p className="text-sm text-destructive">{error}</p> : null}

              <Button type="submit" disabled={!canSubmit} className="w-full h-11 gap-2">
                <LogIn className="h-4 w-4" />
                {isLoading ? 'Signing in…' : 'Sign In'}
              </Button>
            </form>
          </CardContent>
        </Card>

        <p className="mt-6 text-center text-sm text-muted-foreground">
          <Link to="/activate" className="text-primary hover:underline">
            Agent? Activate your device here
          </Link>
        </p>

        <p className="mt-4 text-center text-xs text-muted-foreground">
          Secured government system. Unauthorized access prohibited.
        </p>
      </div>
    </div>
  );
}
