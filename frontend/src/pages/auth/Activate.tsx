import { useEffect, useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ShieldCheck } from 'lucide-react';
import { useAuth } from '@/hooks/auth/use-auth';
import { getDeviceId } from '@/services/device/deviceId';
import { KeyManagement } from '@/services/keyManagement/keyManagement';

function base64EncodeUtf8(input: string) {
  return window.btoa(unescape(encodeURIComponent(input)));
}

export default function Activate() {
  const navigate = useNavigate();
  const { activate, isAuthenticated, user } = useAuth();

  const [badgeId, setBadgeId] = useState('');
  const [activationCode, setActivationCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const canSubmit = !!badgeId.trim() && activationCode.trim().length === 6 && !isLoading;

  useEffect(() => {
    if (isAuthenticated && user) {
      if (user.role === 'admin') {
        navigate('/backoffice');
      } else {
        navigate('/mobile');
      }
    }
  }, [isAuthenticated, user, navigate]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!badgeId.trim()) {
      setError('Badge number is required');
      return;
    }

    if (!activationCode.trim()) {
      setError('OTP code is required');
      return;
    }

    if (!/^\d{6}$/.test(activationCode.trim())) {
      setError('OTP code must be exactly 6 digits');
      return;
    }

    setIsLoading(true);
    try {
      const deviceId = await getDeviceId();
      const { publicKey } = await KeyManagement();
      const publicKeyBase64 = base64EncodeUtf8(JSON.stringify(publicKey));

      const result = await activate({
        badgeId: badgeId.trim(),
        activationCode: activationCode.trim(),
        deviceId,
        publicKeyBase64,
      });

      if (!result.success) {
        setError(result.error || 'Activation failed');
      }
      // Navigation handled by AuthContext state update
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Activation failed');
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
          <p className="mt-1 text-sm text-muted-foreground">
            Intelligent Vehicle Identification & Status System
          </p>
        </div>

        <Card className="w-full">
          <CardHeader>
            <CardTitle className="text-xl">Activate your device</CardTitle>
            <CardDescription>
              Enter your badge number and the 6-digit OTP received by SMS.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="badgeId">Badge number</Label>
                <Input
                  id="badgeId"
                  value={badgeId}
                  onChange={(e) => setBadgeId(e.target.value)}
                  placeholder="e.g. 12345"
                  autoComplete="username"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="activationCode">OTP code</Label>
                <Input
                  id="activationCode"
                  value={activationCode}
                  onChange={(e) => {
                    const next = e.target.value.replace(/\D/g, '').slice(0, 6);
                    setActivationCode(next);
                  }}
                  placeholder="6-digit code"
                  inputMode="numeric"
                  autoComplete="one-time-code"
                  pattern="\d{6}"
                  minLength={6}
                  maxLength={6}
                />
              </div>

              {error ? <p className="text-sm text-destructive">{error}</p> : null}

              {!error && activationCode && activationCode.length < 6 ? (
                <p className="text-xs text-muted-foreground">Enter the 6-digit OTP code.</p>
              ) : null}

              <Button type="submit" disabled={!canSubmit} className="w-full h-11">
                {isLoading ? 'Activating…' : 'Activate'}
              </Button>
            </form>
          </CardContent>
        </Card>

        <p className="mt-6 text-center text-sm text-muted-foreground">
          <Link to="/admin-login" className="text-primary hover:underline">
            Admin? Sign in here
          </Link>
        </p>

        <p className="mt-4 text-center text-xs text-muted-foreground">
          Secured government system. Unauthorized access prohibited.
        </p>
      </div>
    </div>
  );
}
