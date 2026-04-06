import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Building2, Plus, Pencil, Trash2, Search, Loader2 } from 'lucide-react';
import { useOrganizations } from '@/hooks/api/useOrganizations';
import { OrganizationForm } from '@/components/shared/Admin/OrganizationForm';
import { toast } from 'sonner';
import type { Organization } from '@/openapi-rq/requests/types.gen';

export default function OrganizationManagement() {
  const { t } = useTranslation();
  const {
    organizations,
    isLoading,
    createOrganization,
    updateOrganization,
    deleteOrganization,
    isCreating,
    isUpdating,
    isDeleting,
  } = useOrganizations();

  const [searchQuery, setSearchQuery] = useState('');
  const [typeFilter, setTypeFilter] = useState<string>('all');
  const [showForm, setShowForm] = useState(false);
  const [editingOrg, setEditingOrg] = useState<Organization | null>(null);
  const [deletingOrg, setDeletingOrg] = useState<Organization | null>(null);

  // Filter organizations
  const filteredOrgs = (organizations || []).filter((org) => {
    const matchesSearch = org.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesType = typeFilter === 'all' || org.orgType === typeFilter;
    return matchesSearch && matchesType;
  });

  const handleCreate = async (data: any) => {
    try {
      await createOrganization(data);
      toast.success(t('organizationManagement.organizationCreated'));
      setShowForm(false);
    } catch (error: any) {
      toast.error(error.message || t('organizationManagement.createError'));
    }
  };

  const handleUpdate = async (data: any) => {
    if (!editingOrg) return;
    try {
      await updateOrganization(editingOrg.id, data);
      toast.success(t('organizationManagement.organizationUpdated'));
      setEditingOrg(null);
      setShowForm(false);
    } catch (error: any) {
      toast.error(error.message || t('organizationManagement.updateError'));
    }
  };

  const handleDelete = async () => {
    if (!deletingOrg) return;
    try {
      await deleteOrganization(deletingOrg.id);
      toast.success(t('organizationManagement.organizationDeleted'));
      setDeletingOrg(null);
    } catch (error: any) {
      toast.error(error.message || t('organizationManagement.deleteError'));
    }
  };

  const openEditForm = (org: Organization) => {
    setEditingOrg(org);
    setShowForm(true);
  };

  const closeForm = () => {
    setShowForm(false);
    setEditingOrg(null);
  };

  return (
    <BackOfficeLayout
      title={t('organizationManagement.title')}
      subtitle={t('organizationManagement.subtitle')}
      actions={
        <Button onClick={() => setShowForm(true)} className="gap-2">
          <Plus className="h-4 w-4" />
          {t('organizationManagement.addOrganization')}
        </Button>
      }
    >
      <div className="space-y-6">
        {/* Search and Filter */}
        <Card>
          <CardContent className="pt-6">
            <div className="flex flex-col gap-4 sm:flex-row">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder={t('organizationManagement.searchPlaceholder')}
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9"
                />
              </div>
              <Select value={typeFilter} onValueChange={setTypeFilter}>
                <SelectTrigger className="w-full sm:w-[200px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('organizationManagement.allTypes')}</SelectItem>
                  <SelectItem value="police">{t('organizationManagement.types.police')}</SelectItem>
                  <SelectItem value="customs">
                    {t('organizationManagement.types.customs')}
                  </SelectItem>
                  <SelectItem value="border_control">
                    {t('organizationManagement.types.border_control')}
                  </SelectItem>
                  <SelectItem value="other">{t('organizationManagement.types.other')}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        {/* Organizations Table */}
        <Card>
          <CardContent className="p-0">
            {isLoading ? (
              <div className="flex h-64 items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : filteredOrgs.length === 0 ? (
              <div className="flex h-64 flex-col items-center justify-center gap-2 text-muted-foreground">
                <Building2 className="h-12 w-12" />
                <p className="text-sm">
                  {searchQuery || typeFilter !== 'all'
                    ? t('organizationManagement.noResults')
                    : t('organizationManagement.emptyState')}
                </p>
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t('organizationManagement.organizationName')}</TableHead>
                    <TableHead>{t('organizationManagement.organizationType')}</TableHead>
                    <TableHead>{t('organizationManagement.region')}</TableHead>
                    <TableHead className="text-right">
                      {t('organizationManagement.actions')}
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredOrgs.map((org) => (
                    <TableRow key={org.id}>
                      <TableCell className="font-medium">{org.name}</TableCell>
                      <TableCell>
                        <span className="inline-flex items-center rounded-full bg-primary/10 px-2.5 py-0.5 text-xs font-medium text-primary">
                          {t(`organizationManagement.types.${org.orgType}`)}
                        </span>
                      </TableCell>
                      <TableCell>{org.region || '-'}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-2">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => openEditForm(org)}
                            className="h-8 w-8 p-0"
                          >
                            <Pencil className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => setDeletingOrg(org)}
                            className="h-8 w-8 p-0 text-destructive hover:text-destructive"
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Create/Edit Form Dialog */}
      {showForm && (
        <OrganizationForm
          onSubmit={editingOrg ? handleUpdate : handleCreate}
          onCancel={closeForm}
          isLoading={isCreating || isUpdating}
          initialData={editingOrg || undefined}
        />
      )}

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={!!deletingOrg} onOpenChange={() => setDeletingOrg(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('organizationManagement.deleteOrganization')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('organizationManagement.confirmDelete', { name: deletingOrg?.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              disabled={isDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isDeleting ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  {t('common.deleting')}
                </>
              ) : (
                t('common.delete')
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </BackOfficeLayout>
  );
}
