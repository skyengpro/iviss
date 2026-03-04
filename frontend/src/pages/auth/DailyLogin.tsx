import { useState, useEffect, useCallback, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Shield, Clock, Smartphone, CheckCircle2, AlertCircle, Loader2 } from 'lucide-react';
import { useAuth } from '@/hooks/auth/use-auth';
import { getDeviceId } from '@/services/deviceId';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';
const OTP_VALIDITY_SECONDS = 300; // 5 minutes

type Phase = 'request' | 'verify' | 'success';

export default function DailyLogin() {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const { user, isAuthenticated, setShiftSession } = useAuth();

  const [phase, setPhase] = useState<Phase>('request');
  const [otp, setOtp] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [countdown, setCountdown] = useState(0);
  const countdownRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const inputRefs = useRef<(HTMLInputElement | null)[]>([]);

  // Redirect unauthenticated users to login
  useEffect(() => {
    if (!isAuthenticated) {
      navigate('/login');
    }
  }, [isAuthenticated, navigate]);

  // Countdown timer effect
  // Cleanup interval on unmount only
  useEffect(() => {
    return () => {
      if (countdownRef.current) {
        clearInterval(countdownRef.current);
      }
    };
  }, []);

  // Handle OTP expiration when countdown reaches zero
  useEffect(() => {
    if (countdown === 0 && phase === 'verify') {
      setError(t('dailyLogin.otpExpired'));
      setPhase('request');
      setOtp('');
    }
  }, [countdown, phase, t]);

  const startCountdown = useCallback(() => {
    if (countdownRef.current) clearInterval(countdownRef.current);
    setCountdown(OTP_VALIDITY_SECONDS);
    countdownRef.current = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          if (countdownRef.current) {
            clearInterval(countdownRef.current);
            countdownRef.current = null;
          }
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
  }, []);

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getCountdownColor = () => {
    if (countdown > 120) return 'text-emerald-400';
    if (countdown > 60) return 'text-amber-400';
    return 'text-red-400';
  };

  // —— Phase 1: Request OTP ——
  const handleRequestOtp = async () => {
    setError('');
    setIsLoading(true);

    try {
      const deviceId = await getDeviceId();
      const phoneNumber = user?.phoneNumber || '';

      if (!phoneNumber) {
        setError(t('dailyLogin.phoneNotFound'));
        setIsLoading(false);
        return;
      }

      const response = await fetch(`${API_BASE}/auth/request-daily-login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          phone_number: phoneNumber,
          device_id: deviceId,
        }),
      });

      if (!response.ok) {
        const data = await response.json().catch(() => ({ message: '' }));
        const msg = (data.message || '').toLowerCase();

        if (msg.includes('not active')) {
          throw new Error(t('dailyLogin.accountInactive'));
        }
        throw new Error(t('dailyLogin.failedToSend'));
      }

      setPhase('verify');
      startCountdown();
    } catch (err) {
      setError(err instanceof Error ? err.message : t('dailyLogin.failedToRequest'));
    } finally {
      setIsLoading(false);
    }
  };

  // —— Phase 2: Verify OTP ——
  const handleVerifyOtp = async (e: React.FormEvent) => {
    e.preventDefault();
    if (otp.length !== 6) {
      setError(t('dailyLogin.enterComplete'));
      return;
    }

    setError('');
    setIsLoading(true);

    try {
      const deviceId = await getDeviceId();
      const phoneNumber = user?.phoneNumber || '';

      const response = await fetch(`${API_BASE}/auth/verify-daily-login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          phone_number: phoneNumber,
          otp,
          device_id: deviceId,
        }),
      });

      if (!response.ok) {
        const data = await response.json().catch(() => ({ message: '' }));
        const msg = (data.message || '').toLowerCase();

        // Detect max attempts — backend returns "Max attempts reached" only after 5 failures
        if (msg.includes('max attempts')) {
          // Reset to request phase — user must request a new code
          if (countdownRef.current) {
            clearInterval(countdownRef.current);
            countdownRef.current = null;
          }
          setCountdown(0);
          setOtp('');
          setPhase('request');
          throw new Error(t('dailyLogin.maxAttempts'));
        }

        // Try to extract remaining attempts count: "Invalid OTP — 4 attempt(s) remaining"
        const attemptsMatch = msg.match(/(\d+)\s*attempt/);
        if (attemptsMatch) {
          const count = parseInt(attemptsMatch[1], 10);
          throw new Error(t('dailyLogin.invalidOtp', { count }));
        }

        throw new Error(t('dailyLogin.invalidOtp', { count: 0 }));
      }

      const data = await response.json();

      // Store shift token via AuthContext
      setShiftSession(data.access_token, data.expires_in);

      // Brief success state before redirect
      setPhase('success');
      if (countdownRef.current) clearInterval(countdownRef.current);

      setTimeout(() => {
        navigate('/mobile');
      }, 1500);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('dailyLogin.verificationFailed'));
      // Clear OTP input on failure so user can retype
      if (phase === 'verify') {
        setOtp('');
        inputRefs.current[0]?.focus();
      }
    } finally {
      setIsLoading(false);
    }
  };

  // Handle individual digit input for OTP
  const handleOtpChange = (index: number, value: string) => {
    if (!/^\d*$/.test(value)) return;

    const digits = otp.split('');
    while (digits.length < 6) digits.push('');

    if (value.length === 6) {
      // Pasted full code
      setOtp(value);
      inputRefs.current[5]?.focus();
      return;
    }

    digits[index] = value.slice(-1);
    const newOtp = digits.join('');
    setOtp(newOtp);

    // Auto-advance to next input
    if (value && index < 5) {
      inputRefs.current[index + 1]?.focus();
    }
  };

  const handleOtpKeyDown = (index: number, e: React.KeyboardEvent) => {
    if (e.key === 'Backspace' && !otp[index] && index > 0) {
      inputRefs.current[index - 1]?.focus();
    }
  };

  return (
    <div className="min-h-[100dvh] bg-gradient-to-br from-navy-900 via-navy-800 to-navy-900 flex flex-col items-center justify-center p-4">
      {/* Background pattern */}
      <div className="absolute inset-0 opacity-5">
        <div
          className="absolute inset-0"
          style={{
            backgroundImage: `url("data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23ffffff' fill-opacity='1'%3E%3Cpath d='M36 34v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6 34v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6 4V0H4v4H0v2h4v4h2V6h4V4H6z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E")`,
          }}
        />
      </div>

      <div className="w-full max-w-md relative z-10 animate-fade-in">
        {/* Logo */}
        <div className="mb-6 text-center">
          <div className="inline-flex h-14 w-14 items-center justify-center rounded-2xl bg-accent shadow-lg">
            <Shield className="h-8 w-8 text-accent-foreground" />
          </div>
          <h1 className="mt-3 text-2xl font-bold text-white tracking-wide">IVISS</h1>
          <p className="mt-1 text-xs text-white/60 px-4">{t('dailyLogin.pageTitle')}</p>
        </div>

        <Card className="border-0 shadow-2xl glass card-elevated">
          <CardHeader className="space-y-1 pb-4">
            <CardTitle className="text-2xl text-center">
              {phase === 'request' && t('dailyLogin.startShift')}
              {phase === 'verify' && t('dailyLogin.enterOtp')}
              {phase === 'success' && t('dailyLogin.shiftStarted')}
            </CardTitle>
            <CardDescription className="text-center">
              {phase === 'request' && t('dailyLogin.requestDescription')}
              {phase === 'verify' && t('dailyLogin.enterDescription')}
              {phase === 'success' && t('dailyLogin.successDescription')}
            </CardDescription>
          </CardHeader>

          <CardContent>
            {/* Error display */}
            {error && (
              <div className="mb-4 flex items-start gap-2 rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
                <AlertCircle className="h-4 w-4 mt-0.5 shrink-0" />
                <span>{error}</span>
              </div>
            )}

            {/* ── Phase 1: Request OTP ── */}
            {phase === 'request' && (
              <div className="space-y-6">
                <div className="flex flex-col items-center gap-4 py-4">
                  <div className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-accent/10 text-accent">
                    <Smartphone className="h-8 w-8" />
                  </div>
                  <p className="text-sm text-muted-foreground text-center max-w-xs">
                    {t('dailyLogin.smsNotice')}
                  </p>
                </div>

                <Button
                  id="request-otp-button"
                  onClick={handleRequestOtp}
                  className="w-full h-12 bg-accent text-accent-foreground hover:bg-accent/90 text-base font-semibold"
                  disabled={isLoading}
                >
                  {isLoading ? (
                    <Loader2 className="h-5 w-5 animate-spin" />
                  ) : (
                    <>
                      <Clock className="h-5 w-5 mr-2" />
                      {t('dailyLogin.requestOtp')}
                    </>
                  )}
                </Button>
              </div>
            )}

            {/* ── Phase 2: Verify OTP ── */}
            {phase === 'verify' && (
              <form onSubmit={handleVerifyOtp} className="space-y-6">
                {/* Countdown timer */}
                <div className="flex flex-col items-center gap-1">
                  <div
                    className={`flex items-center gap-2 text-2xl font-mono font-bold ${getCountdownColor()} transition-colors`}
                  >
                    <Clock className="h-5 w-5" />
                    {formatTime(countdown)}
                  </div>
                  <p className="text-xs text-muted-foreground">{t('dailyLogin.timeRemaining')}</p>
                </div>

                {/* OTP digit inputs */}
                <div className="space-y-2">
                  <Label className="text-center block">{t('dailyLogin.verificationCode')}</Label>
                  <div className="flex justify-center gap-2">
                    {Array.from({ length: 6 }).map((_, i) => (
                      <Input
                        key={i}
                        id={`otp-digit-${i}`}
                        ref={(el) => {
                          inputRefs.current[i] = el;
                        }}
                        type="text"
                        inputMode="numeric"
                        maxLength={i === 0 ? 6 : 1}
                        value={otp[i] || ''}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                          handleOtpChange(i, e.target.value)
                        }
                        onKeyDown={(e: React.KeyboardEvent) => handleOtpKeyDown(i, e)}
                        className="w-11 h-13 text-center text-xl font-bold tracking-wide"
                        autoFocus={i === 0}
                      />
                    ))}
                  </div>
                </div>

                <Button
                  id="verify-otp-button"
                  type="submit"
                  className="w-full h-12 bg-accent text-accent-foreground hover:bg-accent/90 text-base font-semibold"
                  disabled={isLoading || otp.length !== 6 || countdown === 0}
                >
                  {isLoading ? (
                    <Loader2 className="h-5 w-5 animate-spin" />
                  ) : (
                    t('dailyLogin.verifyAndStart')
                  )}
                </Button>

                <button
                  type="button"
                  onClick={() => {
                    setPhase('request');
                    setOtp('');
                    setError('');
                    if (countdownRef.current) clearInterval(countdownRef.current);
                    setCountdown(0);
                  }}
                  className="w-full text-center text-sm text-muted-foreground hover:text-accent transition-colors"
                >
                  {t('dailyLogin.requestNewCode')}
                </button>
              </form>
            )}

            {/* ── Phase 3: Success ── */}
            {phase === 'success' && (
              <div className="flex flex-col items-center gap-4 py-6">
                <div className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-emerald-500/10 text-emerald-500 animate-bounce">
                  <CheckCircle2 className="h-10 w-10" />
                </div>
                <p className="text-sm text-muted-foreground">{t('dailyLogin.shiftActive')}</p>
              </div>
            )}

            {/* Security notice */}
            <div className="mt-6 rounded-lg bg-muted/30 p-2.5 text-center text-[10px] leading-tight text-muted-foreground/80">
              <Shield className="inline h-3 w-3 mr-1 opacity-70" />
              {t('dailyLogin.securityNotice')}
            </div>
          </CardContent>
        </Card>

        {/* Footer */}
        <p className="mt-4 text-center text-[10px] text-white/30">{t('dailyLogin.footer')}</p>
      </div>
    </div>
  );
}
