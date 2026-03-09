import { MobileLayout } from '@/components/layout/MobileLayout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function MobileSettings() {
  return (
    <MobileLayout title="Settings">
      <div className="p-4">
        <Card>
          <CardHeader>
            <CardTitle>Settings</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">Settings screen coming soon.</p>
          </CardContent>
        </Card>
      </div>
    </MobileLayout>
  );
}
