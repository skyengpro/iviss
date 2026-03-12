import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { ShieldCheck, Building2 } from 'lucide-react';
import { useAuth } from '@/hooks/auth/use-auth';
import { getDeviceId } from '@/services/deviceId';

export default function DailyLogin() {
  const navigate = useNavigate();
  const { dailyLoginRequest, dailyLoginVerify, isAuthenticated, user, login } = useAuth();

  const [badgeId, setBadgeId] = useState('');
  const [activationCode, setActivationCode] = useState('');
  const [step, setStep] = useState<'REQUEST' | 'VERIFY'>('REQUEST');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const canRequest = !!badgeId.trim() && !isLoading;
  const canVerify = !!badgeId.trim() && activationCode.trim().length === 6 && !isLoading;

  useEffect(() => {
    // Redirect if already authenticated and shift is active
    if (isAuthenticated && user) {
      if (user.role === 'admin') {
        navigate('/backoffice');
      } else {
        navigate('/mobile');
      }
    }
  }, [isAuthenticated, user, navigate]);

  const handleRequestOTP = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!badgeId.trim()) {
      setError('Badge number is required');
      return;
    }

    setIsLoading(true);
    try {
      const result = await dailyLoginRequest({ badgeId: badgeId.trim() });
      if (!result.success) {
        setError(result.error || 'Failed to request OTP');
      } else {
        setStep('VERIFY');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to request OTP');
    } finally {
      setIsLoading(false);
    }
  };

  const handleVerifyOTP = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!badgeId.trim()) {
      setError('Badge number is required');
      return;
    }

    if (!activationCode.trim() || !/^\d{6}$/.test(activationCode.trim())) {
      setError('OTP code must be exactly 6 digits');
      return;
    }

    setIsLoading(true);
    try {
      const deviceId = await getDeviceId();
      const result = await dailyLoginVerify({
        badgeId: badgeId.trim(),
        activationCode: activationCode.trim(),
        deviceId,
      });

      if (!result.success) {
        setError(result.error || 'Verification failed');
      }
      // Navigation is handled by the useEffect above triggered by auth state change
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed');
    } finally {
      setIsLoading(false);
    }
  };

  const handleAdminLogin = async () => {
    setIsLoading(true);
    const result = await login('admin01', 'admin123');
    setIsLoading(false);

    if (result.success) {
      navigate('/backoffice');
    } else {
      setError(result.error || 'Admin login failed');
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
            <CardTitle className="text-xl">Daily Agent Login</CardTitle>
            <CardDescription>
              {step === 'REQUEST'
                ? 'Enter your badge number to receive a one-time password for your shift.'
                : 'Enter your badge number and the 6-digit OTP sent to your phone.'}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {step === 'REQUEST' ? (
              <form onSubmit={handleRequestOTP} className="space-y-4">
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
                {error && <p className="text-sm text-destructive">{error}</p>}
                <Button type="submit" disabled={!canRequest} className="w-full h-11">
                  {isLoading ? 'Requesting...' : 'Request OTP'}
                </Button>
              </form>
            ) : (
              <form onSubmit={handleVerifyOTP} className="space-y-4">
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
                  <Label htmlFor="activationCode">OTP Code</Label>
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
                    maxLength={6}
                  />
                </div>
                {error && <p className="text-sm text-destructive">{error}</p>}
                {!error && activationCode.length > 0 && activationCode.length < 6 && (
                  <p className="text-xs text-muted-foreground">Enter the 6-digit OTP code.</p>
                )}
                <Button type="submit" disabled={!canVerify} className="w-full h-11">
                  {isLoading ? 'Verifying...' : 'Submit'}
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  className="w-full mt-2"
                  onClick={() => {
                    setStep('REQUEST');
                    setActivationCode('');
                    setError('');
                  }}
                  disabled={isLoading}
                >
                  Back
                </Button>
              </form>
            )}

            <div className="pt-4 pb-2">
              <Separator />
            </div>

            <Button
              type="button"
              variant="outline"
              onClick={handleAdminLogin}
              className="w-full h-11 gap-2"
              disabled={isLoading}
            >
              <Building2 className="h-4 w-4" />
              Admin login
            </Button>
          </CardContent>
        </Card>

        <p className="mt-6 text-center text-xs text-muted-foreground">
          Secured government system. Unauthorized access prohibited.
        </p>
      </div>
    </div>
  );
}
