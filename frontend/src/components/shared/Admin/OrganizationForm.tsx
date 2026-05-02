import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { useTranslation } from 'react-i18next';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Loader2 } from 'lucide-react';
import type {
  CreateOrganizationRequest,
  Organization,
  UpdateOrganizationRequest,
} from '@/openapi-rq/requests/types.gen';

function minutesToTimeValue(minutes?: number) {
  if (minutes === undefined || minutes === null) return '';

  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;

  return `${String(hours).padStart(2, '0')}:${String(mins).padStart(2, '0')}`;
}

function timeValueToMinutes(value?: string) {
  if (!value) return undefined;

  const [hours, minutes] = value.split(':').map(Number);

  if (Number.isNaN(hours) || Number.isNaN(minutes)) return undefined;

  return hours * 60 + minutes;
}

const formSchema = z
  .object({
    name: z.string().min(1, 'Name is required').max(255, 'Name is too long'),
    orgType: z.enum(['police', 'customs', 'border_control', 'other']),
    region: z.string().max(100, 'Region is too long').optional(),
    startWorkTime: z.string().optional(),
    endWorkTime: z.string().optional(),
  })
  .superRefine((values, ctx) => {
    const startMinutes = timeValueToMinutes(values.startWorkTime);
    const endMinutes = timeValueToMinutes(values.endWorkTime);

    if (values.startWorkTime && startMinutes === undefined) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'Invalid start work time',
        path: ['startWorkTime'],
      });
    }

    if (values.endWorkTime && endMinutes === undefined) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'Invalid end work time',
        path: ['endWorkTime'],
      });
    }

    if (startMinutes !== undefined && endMinutes !== undefined && startMinutes >= endMinutes) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'End work time must be after start work time',
        path: ['endWorkTime'],
      });
    }
  });

type FormValues = z.infer<typeof formSchema>;

interface OrganizationFormProps {
  onSubmit: (data: CreateOrganizationRequest | UpdateOrganizationRequest) => Promise<void>;
  onCancel: () => void;
  isLoading?: boolean;
  initialData?: Organization;
}

export function OrganizationForm({
  onSubmit,
  onCancel,
  isLoading,
  initialData,
}: OrganizationFormProps) {
  const { t } = useTranslation();

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: initialData?.name || '',
      orgType: initialData?.orgType || 'police',
      region: initialData?.region || '',
      startWorkTime: minutesToTimeValue(initialData?.startWorkTime),
      endWorkTime: minutesToTimeValue(initialData?.endWorkTime),
    },
  });

  const handleSubmit = async (data: FormValues) => {
    await onSubmit({
      name: data.name,
      orgType: data.orgType,
      region: data.region || undefined,
      startWorkTime: timeValueToMinutes(data.startWorkTime),
      endWorkTime: timeValueToMinutes(data.endWorkTime),
    });
  };

  return (
    <Dialog open onOpenChange={onCancel}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>
            {initialData
              ? t('organizationManagement.editOrganization')
              : t('organizationManagement.addOrganization')}
          </DialogTitle>
          <DialogDescription>
            {initialData
              ? t('organizationManagement.editDescription')
              : t('organizationManagement.createDescription')}
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>{t('organizationManagement.organizationName')}</FormLabel>
                  <FormControl>
                    <Input
                      placeholder={t('organizationManagement.namePlaceholder')}
                      {...field}
                      disabled={isLoading}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="orgType"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>{t('organizationManagement.organizationType')}</FormLabel>
                  <Select
                    onValueChange={field.onChange}
                    defaultValue={field.value}
                    disabled={isLoading}
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="police">
                        {t('organizationManagement.types.police')}
                      </SelectItem>
                      <SelectItem value="customs">
                        {t('organizationManagement.types.customs')}
                      </SelectItem>
                      <SelectItem value="border_control">
                        {t('organizationManagement.types.border_control')}
                      </SelectItem>
                      <SelectItem value="other">
                        {t('organizationManagement.types.other')}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="region"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    {t('organizationManagement.region')}{' '}
                    <span className="text-muted-foreground">({t('common.optional')})</span>
                  </FormLabel>
                  <FormControl>
                    <Input
                      placeholder={t('organizationManagement.regionPlaceholder')}
                      {...field}
                      disabled={isLoading}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="grid gap-4 sm:grid-cols-2">
              <FormField
                control={form.control}
                name="startWorkTime"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>{t('organizationManagement.startWorkTime')}</FormLabel>
                    <FormControl>
                      <Input
                        type="time"
                        step={60}
                        value={field.value ?? ''}
                        onChange={field.onChange}
                        disabled={isLoading}
                      />
                    </FormControl>
                    <FormDescription>
                      {t('organizationManagement.defaultStartWorkTimeHint')}
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="endWorkTime"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>{t('organizationManagement.endWorkTime')}</FormLabel>
                    <FormControl>
                      <Input
                        type="time"
                        step={60}
                        value={field.value ?? ''}
                        onChange={field.onChange}
                        disabled={isLoading}
                      />
                    </FormControl>
                    <FormDescription>
                      {t('organizationManagement.defaultEndWorkTimeHint')}
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <DialogFooter>
              <Button type="button" variant="outline" onClick={onCancel} disabled={isLoading}>
                {t('common.cancel')}
              </Button>
              <Button type="submit" disabled={isLoading}>
                {isLoading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {t('common.saving')}
                  </>
                ) : initialData ? (
                  t('common.update')
                ) : (
                  t('common.create')
                )}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
