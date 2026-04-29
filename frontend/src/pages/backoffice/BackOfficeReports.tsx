import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function BackOfficeReports() {
  const { t } = useTranslation();

  return (
    <BackOfficeLayout title={t('backOfficeSidebar.generateReport')} subtitle="">
      <div className="space-y-6">
        <Card className="rounded-2xl border border-border/60 bg-card shadow-md">
          <CardHeader>
            <CardTitle>{t('backOfficeSidebar.generateReport')}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-sm text-muted-foreground">{t('common.comingSoon')}</div>
          </CardContent>
        </Card>
      </div>
    </BackOfficeLayout>
  );
}
