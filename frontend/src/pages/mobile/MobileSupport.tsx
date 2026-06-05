import { MobileLayout } from '@/components/layout/MobileLayout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function MobileSupport() {
  return (
    <MobileLayout title="Help & Support">
      <div className="p-4">
        <Card>
          <CardHeader>
            <CardTitle>Help & Support</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              If you need assistance, contact your supervisor or the IVISS support team.
            </p>
          </CardContent>
        </Card>
      </div>
    </MobileLayout>
  );
}
