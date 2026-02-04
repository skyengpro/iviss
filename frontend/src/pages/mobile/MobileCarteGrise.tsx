import { useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { MobileLayout } from '@/components/layout/MobileLayout';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Camera, ArrowLeft, CheckCircle, Upload, FileText } from 'lucide-react';
import { useAuth } from '@/hooks/use-auth';
import { mockVehicleService } from '@/services/mockVehicles';
import { toast } from '@/hooks/use-toast';

type CaptureStep = 'front' | 'back' | 'review' | 'submitted';

export default function MobileCarteGrise() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { user } = useAuth();
  const { t } = useTranslation();
  const plateNumber = searchParams.get('plate') || '';

  const [step, setStep] = useState<CaptureStep>('front');
  const [frontImage, setFrontImage] = useState<string | null>(null);
  const [backImage, setBackImage] = useState<string | null>(null);
  const [notes, setNotes] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleCapture = (side: 'front' | 'back') => {
    // Simulate capture
    const mockImage = '/placeholder.svg';

    if (side === 'front') {
      setFrontImage(mockImage);
      setStep('back');
    } else {
      setBackImage(mockImage);
      setStep('review');
    }
  };

  const handleSubmit = async () => {
    if (!user || !frontImage || !backImage) return;

    setIsSubmitting(true);

    try {
      await mockVehicleService.submitPendingVehicle({
        plateNumber,
        submittedBy: user.name,
        location: 'Highway A1, KM 42',
        frontImage,
        backImage,
        notes: notes || undefined,
      });

      setStep('submitted');

      toast({
        title: t('mobileCarteGrise.toastSuccessTitle'),
        description: t('mobileCarteGrise.toastSuccessDescription'),
      });
    } catch (error) {
      toast({
        title: t('mobileCarteGrise.toastErrorTitle'),
        description: t('mobileCarteGrise.toastErrorDescription'),
        variant: 'destructive',
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  if (step === 'submitted') {
    return (
      <MobileLayout title={t('mobileCarteGrise.submissionCompleteTitle')} hideNavigation>
        <div className="flex flex-col items-center justify-center min-h-[60vh] p-4 text-center">
          <div className="rounded-full bg-status-valid/10 p-6">
            <CheckCircle className="h-16 w-16 text-status-valid" />
          </div>
          <h2 className="mt-6 text-xl font-bold">{t('mobileCarteGrise.submittedSuccessfully')}</h2>
          <p className="mt-2 text-muted-foreground">
            {t('mobileCarteGrise.submissionMessage', { plateNumber })}
          </p>
          <p className="mt-4 text-sm text-muted-foreground">
            {t('mobileCarteGrise.status')}{' '}
            <span className="text-status-pending font-medium">
              {t('mobileCarteGrise.pendingValidation')}
            </span>
          </p>

          <div className="mt-8 w-full space-y-2">
            <Button
              className="w-full bg-accent text-accent-foreground hover:bg-accent/90"
              onClick={() => navigate('/mobile')}
            >
              {t('mobileCarteGrise.returnToDashboard')}
            </Button>
            <Button variant="outline" className="w-full" onClick={() => navigate('/mobile/search')}>
              {t('mobileCarteGrise.newSearch')}
            </Button>
          </div>
        </div>
      </MobileLayout>
    );
  }

  return (
    <MobileLayout title={t('mobileCarteGrise.captureTitle')} hideNavigation>
      <div className="p-4 space-y-6">
        <button
          onClick={() => {
            if (step === 'front') navigate(-1);
            else if (step === 'back') setStep('front');
            else if (step === 'review') setStep('back');
          }}
          className="flex items-center gap-2 text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="h-4 w-4" />
          {t('mobileCarteGrise.backButton')}
        </button>

        {/* Header */}
        <div className="rounded-xl bg-muted p-4">
          <p className="text-sm text-muted-foreground">
            {t('mobileCarteGrise.registeringVehicle')}
          </p>
          <p className="text-lg font-mono font-bold tracking-widest">{plateNumber}</p>
        </div>

        {/* Progress indicator */}
        <div className="flex items-center justify-center gap-2">
          <StepIndicator
            number={1}
            label={t('mobileCarteGrise.stepFront')}
            active={step === 'front'}
            completed={frontImage !== null}
          />
          <div className="w-8 h-0.5 bg-border" />
          <StepIndicator
            number={2}
            label={t('mobileCarteGrise.stepBack')}
            active={step === 'back'}
            completed={backImage !== null}
          />
          <div className="w-8 h-0.5 bg-border" />
          <StepIndicator
            number={3}
            label={t('mobileCarteGrise.stepReview')}
            active={step === 'review'}
            completed={false}
          />
        </div>

        {/* Content based on step */}
        {step === 'front' && (
          <CaptureCard
            title={t('mobileCarteGrise.frontTitle')}
            description={t('mobileCarteGrise.frontDescription')}
            onCapture={() => handleCapture('front')}
            image={frontImage}
          />
        )}

        {step === 'back' && (
          <CaptureCard
            title={t('mobileCarteGrise.backTitle')}
            description={t('mobileCarteGrise.backDescription')}
            onCapture={() => handleCapture('back')}
            image={backImage}
          />
        )}

        {step === 'review' && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">{t('mobileCarteGrise.reviewSubmission')}</h2>
            <div className="grid grid-cols-2 gap-4">
              <div className="rounded-lg border border-border overflow-hidden">
                <div className="bg-muted p-2 text-center text-xs font-medium">
                  {t('mobileCarteGrise.stepFront')}
                </div>
                <div className="aspect-[3/4] bg-muted flex items-center justify-center">
                  <FileText className="h-12 w-12 text-muted-foreground/50" />
                </div>
              </div>
              <div className="rounded-lg border border-border overflow-hidden">
                <div className="bg-muted p-2 text-center text-xs font-medium">
                  {t('mobileCarteGrise.stepBack')}
                </div>
                <div className="aspect-[3/4] bg-muted flex items-center justify-center">
                  <FileText className="h-12 w-12 text-muted-foreground/50" />
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">{t('mobileCarteGrise.notesLabel')}</label>
              <Textarea
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                placeholder={t('mobileCarteGrise.notesPlaceholder')}
                rows={3}
              />
            </div>

            <Button
              className="w-full h-12 gap-2 bg-accent text-accent-foreground hover:bg-accent/90"
              onClick={handleSubmit}
              disabled={isSubmitting}
            >
              {isSubmitting ? (
                <div className="h-5 w-5 animate-spin rounded-full border-2 border-current border-t-transparent" />
              ) : (
                <>
                  <Upload className="h-5 w-5" />
                  {t('mobileCarteGrise.submitButton')}
                </>
              )}
            </Button>
          </div>
        )}
      </div>
    </MobileLayout>
  );
}

function StepIndicator({
  number,
  label,
  active,
  completed,
}: {
  number: number;
  label: string;
  active: boolean;
  completed: boolean;
}) {
  return (
    <div className="flex flex-col items-center">
      <div
        className={`
        flex h-8 w-8 items-center justify-center rounded-full text-sm font-semibold
        ${
          completed
            ? 'bg-status-valid text-status-valid-foreground'
            : active
              ? 'bg-accent text-accent-foreground'
              : 'bg-muted text-muted-foreground'
        }
      `}
      >
        {completed ? <CheckCircle className="h-4 w-4" /> : number}
      </div>
      <p className={`mt-1 text-xs ${active ? 'text-foreground' : 'text-muted-foreground'}`}>
        {label}
      </p>
    </div>
  );
}

function CaptureCard({
  title,
  description,
  onCapture,
  image,
}: {
  title: string;
  description: string;
  onCapture: () => void;
  image: string | null;
}) {
  const { t } = useTranslation();
  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-lg font-semibold">{title}</h2>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>

      <div className="rounded-xl border-2 border-dashed border-border bg-muted/50 p-8">
        <div className="flex flex-col items-center justify-center">
          <div className="rounded-full bg-accent/10 p-4">
            <Camera className="h-8 w-8 text-accent" />
          </div>
          <p className="mt-4 text-sm text-muted-foreground text-center">
            {t('mobileCarteGrise.captureInstruction')}
          </p>
        </div>
      </div>

      <Button
        className="w-full h-12 gap-2 bg-accent text-accent-foreground hover:bg-accent/90"
        onClick={onCapture}
      >
        <Camera className="h-5 w-5" />
        {t('mobileCarteGrise.captureImageButton')}
      </Button>
    </div>
  );
}
