import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { useTranslation } from 'react-i18next';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { ProvisionUserRequest, UserRole } from '@/openapi-rq/requests/types.gen';
import { Loader2 } from 'lucide-react';
import { useOrganizations } from '@/hooks/api/useOrganizations';

const formSchema = z.object({
  username: z.string().min(3, { message: 'tooShort' }),
  fullName: z
    .string()
    .min(1, { message: 'required' })
    .regex(/^[a-zA-Z\s]*$/, { message: 'invalidName' }),
  phoneNumber: z
    .string()
    .min(1, { message: 'required' })
    .regex(/^\+237\d{8,12}$/, { message: 'invalidPhone' }),
  role: z.enum(['admin', 'agent', 'manager'] as const),
  organizationId: z.string().min(1, { message: 'required' }).uuid({ message: 'invalidUuid' }),
  email: z.string().email({ message: 'invalidEmail' }).optional().or(z.literal('')),
  badgeId: z.string().optional(),
});

type FormValues = z.infer<typeof formSchema>;

interface UserFormProps {
  onSubmit: (data: ProvisionUserRequest) => Promise<void>;
  onCancel: () => void;
  isLoading?: boolean;
  initialData?: Partial<ProvisionUserRequest>;
}

export function UserForm({ onSubmit, onCancel, isLoading, initialData }: UserFormProps) {
  const { t } = useTranslation();
  const { organizations, isLoading: isLoadingOrgs } = useOrganizations();

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      username: initialData?.username || '',
      fullName: initialData?.fullName || '',
      phoneNumber: initialData?.phoneNumber || '+237',
      role: initialData?.role || 'agent',
      organizationId: initialData?.organizationId || '',
      email: initialData?.email || '',
      badgeId: initialData?.badgeId || '',
    },
  });

  const handleSubmit = async (values: FormValues) => {
    const payload: ProvisionUserRequest = {
      username: values.username,
      fullName: values.fullName,
      phoneNumber: values.phoneNumber,
      role: values.role as UserRole,
      organizationId: values.organizationId,
      email: values.email || undefined,
      badgeId: values.badgeId || undefined,
    };
    await onSubmit(payload);
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <FormField
            control={form.control}
            name="username"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t('backOfficeUserManagement.username')}</FormLabel>
                <FormControl>
                  <Input {...field} />
                </FormControl>
                <FormMessage>
                  {form.formState.errors.username?.message &&
                    t(
                      `backOfficeUserManagement.validation.${form.formState.errors.username.message}`,
                      { count: 3 }
                    )}
                </FormMessage>
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="fullName"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t('backOfficeUserManagement.fullName')}</FormLabel>
                <FormControl>
                  <Input
                    {...field}
                    onChange={(e) => {
                      const value = e.target.value.replace(/[^a-zA-Z\s]/g, '');
                      field.onChange(value);
                    }}
                  />
                </FormControl>
                <FormMessage>
                  {form.formState.errors.fullName?.message &&
                    t(
                      `backOfficeUserManagement.validation.${form.formState.errors.fullName.message}`
                    )}
                </FormMessage>
              </FormItem>
            )}
          />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <FormField
            control={form.control}
            name="phoneNumber"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t('backOfficeUserManagement.phoneNumber')}</FormLabel>
                <FormControl>
                  <Input
                    placeholder="+237..."
                    {...field}
                    onChange={(e) => {
                      let value = e.target.value;
                      if (!value.startsWith('+237')) {
                        value = '+237' + value.replace(/\D/g, '');
                      } else {
                        const prefix = '+237';
                        const rest = value.slice(prefix.length).replace(/\D/g, '');
                        value = prefix + rest;
                      }
                      field.onChange(value);
                    }}
                  />
                </FormControl>
                <FormMessage>
                  {form.formState.errors.phoneNumber?.message &&
                    t(
                      `backOfficeUserManagement.validation.${form.formState.errors.phoneNumber.message}`
                    )}
                </FormMessage>
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="email"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t('backOfficeUserManagement.email')}</FormLabel>
                <FormControl>
                  <Input type="email" {...field} />
                </FormControl>
                <FormMessage>
                  {form.formState.errors.email?.message &&
                    t(`backOfficeUserManagement.validation.${form.formState.errors.email.message}`)}
                </FormMessage>
              </FormItem>
            )}
          />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <FormField
            control={form.control}
            name="role"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t('backOfficeUserManagement.role')}</FormLabel>
                <Select onValueChange={field.onChange} defaultValue={field.value}>
                  <FormControl>
                    <SelectTrigger>
                      <SelectValue placeholder={t('backOfficeUserManagement.role')} />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    <SelectItem value="admin">
                      {t('backOfficeUserManagement.super_admin')}
                    </SelectItem>
                    <SelectItem value="manager">
                      {t('backOfficeUserManagement.supervisor')}
                    </SelectItem>
                    <SelectItem value="agent">{t('backOfficeUserManagement.agent')}</SelectItem>
                  </SelectContent>
                </Select>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="organizationId"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t('backOfficeUserManagement.organization')}</FormLabel>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value}
                  disabled={isLoadingOrgs}
                >
                  <FormControl>
                    <SelectTrigger>
                      <SelectValue
                        placeholder={
                          isLoadingOrgs
                            ? t('backOfficeUserManagement.loading')
                            : t('backOfficeUserManagement.selectOrganization')
                        }
                      />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    {organizations?.map((org) => (
                      <SelectItem key={org.id} value={org.id}>
                        {org.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <FormMessage>
                  {form.formState.errors.organizationId?.message &&
                    t(
                      `backOfficeUserManagement.validation.${form.formState.errors.organizationId.message}`
                    )}
                </FormMessage>
              </FormItem>
            )}
          />
        </div>

        <FormField
          control={form.control}
          name="badgeId"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t('backOfficeUserManagement.badgeId')}</FormLabel>
              <FormControl>
                <Input {...field} />
              </FormControl>
              <FormMessage>
                {form.formState.errors.badgeId?.message &&
                  t(`backOfficeUserManagement.validation.${form.formState.errors.badgeId.message}`)}
              </FormMessage>
            </FormItem>
          )}
        />

        <div className="flex justify-end gap-2 pt-4">
          <Button type="button" variant="outline" onClick={onCancel} disabled={isLoading}>
            {t('backOfficeUserManagement.cancel')}
          </Button>
          <Button type="submit" className="bg-accent text-accent-foreground" disabled={isLoading}>
            {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('backOfficeUserManagement.saveUser')}
          </Button>
        </div>
      </form>
    </Form>
  );
}
