import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BackOfficeLayout } from '@/components/layout/BackOfficeLayout';
import { StatusBadge } from '@/components/ui/status-badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import {
  Search,
  Plus,
  MoreVertical,
  Shield,
  UserCheck,
  UserX,
  Key,
  Edit,
  Trash2,
  Loader2,
  RefreshCw,
} from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';

import { useUsers } from '@/hooks/api/useUsers';
import { useOrganizations } from '@/hooks/api/useOrganizations';
import { UserForm } from '@/components/shared/Admin/UserForm';
import { toast } from 'sonner';
import { fetchWithAuth } from '@/services/api/backendFetch';
import {
  resendActivationCode,
  terminateSession,
  restartSession,
} from '@/openapi-rq/requests/services.gen';
import {
  UserProfile,
  UpdateUserRequest,
  ProvisionUserRequest,
} from '@/openapi-rq/requests/types.gen';

const roleColors: Record<string, 'default' | 'primary' | 'secondary' | 'destructive' | 'outline'> =
  {
    admin: 'destructive',
    supervisor: 'secondary',
    agent: 'outline',
  };

export default function UserManagement() {
  const { t } = useTranslation();
  const [searchQuery, setSearchQuery] = useState('');
  const [roleFilter, setRoleFilter] = useState('all');
  const [statusFilter, setStatusFilter] = useState('all');
  const [isAddUserOpen, setIsAddUserOpen] = useState(false);
  const [isEditUserOpen, setIsEditUserOpen] = useState(false);
  const [isDeleteConfirmOpen, setIsDeleteConfirmOpen] = useState(false);
  const [isTerminateConfirmOpen, setIsTerminateConfirmOpen] = useState(false);
  const [isRestartConfirmOpen, setIsRestartConfirmOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<UserProfile | null>(null);
  const [resendLoadingUserId, setResendLoadingUserId] = useState<string | null>(null);

  const {
    users = [],
    isLoadingUsers,
    provision,
    isProvisioning,
    update,
    isUpdating,
    remove,
    isDeleting,
  } = useUsers();
  const { organizations = [] } = useOrganizations();

  // Calculate dynamic stats
  const totalUsersCount = users.length;
  const activeNowCount = users.filter((u: UserProfile) => u.status === 'ACTIVE').length;
  const supervisorsCount = users.filter((u: UserProfile) => u.role === 'manager').length;
  const organizationsCount = organizations.length;

  const handleAddUser = async (data: ProvisionUserRequest) => {
    try {
      await provision(data);
      toast.success(t('backOfficeUserManagement.toastSuccess'));
      setIsAddUserOpen(false);
    } catch (error) {
      toast.error(t('backOfficeUserManagement.toastError'));
    }
  };

  const handleEditUser = async (data: ProvisionUserRequest) => {
    if (!selectedUser) return;
    try {
      const updateData: UpdateUserRequest = {
        username: data.username,
        fullName: data.fullName,
        phoneNumber: data.phoneNumber,
        organizationId: data.organizationId,
        email: data.email,
        badgeId: data.badgeId,
        role: data.role,
      };
      await update(selectedUser.id, updateData);
      toast.success(t('backOfficeUserManagement.editSuccess'));
      setIsEditUserOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(t('backOfficeUserManagement.editError'));
    }
  };

  const handleDeleteUser = async () => {
    if (!selectedUser) return;
    try {
      await remove(selectedUser.id);
      toast.success(t('backOfficeUserManagement.deleteSuccess'));
      setIsDeleteConfirmOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(t('backOfficeUserManagement.deleteError'));
    }
  };

  const handleTerminateSession = async () => {
    if (!selectedUser) return;
    try {
      await terminateSession({
        body: { userId: selectedUser.id },
        throwOnError: true,
      });

      toast.success(t('backOfficeUserManagement.terminateSuccess'));
      setIsTerminateConfirmOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(t('backOfficeUserManagement.terminateError'));
    }
  };

  const handleRestartSession = async () => {
    if (!selectedUser) return;
    try {
      await restartSession({
        body: { userId: selectedUser.id },
        throwOnError: true,
      });

      toast.success(t('backOfficeUserManagement.restartSuccess'));
      setIsRestartConfirmOpen(false);
      setSelectedUser(null);
    } catch (error) {
      toast.error(t('backOfficeUserManagement.restartError'));
    }
  };

  const handleResendActivationCode = async (user: UserProfile) => {
    setResendLoadingUserId(user.id);
    try {
      const res = await resendActivationCode({
        body: { userId: user.id },
        throwOnError: false,
      });

      if (res.error) {
        const msg =
          typeof (res.error as { message?: unknown })?.message === 'string'
            ? String((res.error as { message?: unknown }).message)
            : 'Failed to resend activation code';
        toast.error(msg);
        return;
      }

      toast.success(res.data?.message || 'Activation code sent');
    } catch (e) {
      toast.error(e instanceof Error ? e.message : 'Failed to resend activation code');
    } finally {
      setResendLoadingUserId(null);
    }
  };

  const toggleStatus = async (user: UserProfile) => {
    try {
      await update(user.id, { status: user.isActive ? 'SUSPENDED' : 'ACTIVE' });
      toast.success(t('backOfficeUserManagement.toastSuccess'));
    } catch (error) {
      toast.error(t('backOfficeUserManagement.toastError'));
    }
  };

  const filteredUsers = (users as UserProfile[]).filter((user: UserProfile) => {
    const matchesSearch =
      user.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (user.email?.toLowerCase() || '').includes(searchQuery.toLowerCase()) ||
      (user.organization?.toLowerCase() || '').includes(searchQuery.toLowerCase());

    const matchesRole = roleFilter === 'all' || user.role === roleFilter;
    const matchesStatus =
      statusFilter === 'all' ||
      (statusFilter === 'active' && user.status === 'ACTIVE') ||
      (statusFilter === 'pending' && user.status === 'PENDING_ACTIVATION') ||
      (statusFilter === 'suspended' && user.status === 'SUSPENDED');

    return matchesSearch && matchesRole && matchesStatus;
  });

  return (
    <BackOfficeLayout
      title={t('backOfficeUserManagement.title')}
      subtitle={t('backOfficeUserManagement.subtitle')}
      actions={
        <>
          <Dialog open={isAddUserOpen} onOpenChange={setIsAddUserOpen}>
            <DialogTrigger asChild>
              <Button className="gap-2 bg-accent text-accent-foreground hover:bg-accent/90">
                <Plus className="h-4 w-4" />
                {t('backOfficeUserManagement.addUser')}
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[600px]">
              <DialogHeader>
                <DialogTitle>{t('backOfficeUserManagement.addUserTitle')}</DialogTitle>
                <DialogDescription>
                  {t('backOfficeUserManagement.addUserDescription')}
                </DialogDescription>
              </DialogHeader>
              <UserForm
                onSubmit={handleAddUser}
                onCancel={() => setIsAddUserOpen(false)}
                isLoading={isProvisioning}
              />
            </DialogContent>
          </Dialog>

          <Dialog open={isEditUserOpen} onOpenChange={setIsEditUserOpen}>
            <DialogContent className="sm:max-w-[600px]">
              <DialogHeader>
                <DialogTitle>{t('backOfficeUserManagement.editUser')}</DialogTitle>
                <DialogDescription>
                  Update the professional details and access level for this user.
                </DialogDescription>
              </DialogHeader>
              {selectedUser && (
                <UserForm
                  initialData={{
                    username: selectedUser.username,
                    fullName: selectedUser.name,
                    phoneNumber: selectedUser.phoneNumber || '',
                    organizationId: selectedUser.organizationId,
                    role: selectedUser.role,
                    email: selectedUser.email || '',
                    badgeId: selectedUser.badgeId || '',
                  }}
                  onSubmit={handleEditUser}
                  onCancel={() => {
                    setIsEditUserOpen(false);
                    setSelectedUser(null);
                  }}
                  isLoading={isUpdating}
                />
              )}
            </DialogContent>
          </Dialog>

          <AlertDialog open={isDeleteConfirmOpen} onOpenChange={setIsDeleteConfirmOpen}>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{t('backOfficeUserManagement.deleteUser')}?</AlertDialogTitle>
                <AlertDialogDescription>
                  {t('backOfficeUserManagement.deleteDescription') ||
                    'This action cannot be undone. This will permanently remove the user from the active directory.'}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel onClick={() => setSelectedUser(null)}>
                  {t('backOfficeUserManagement.cancel')}
                </AlertDialogCancel>
                <AlertDialogAction
                  onClick={handleDeleteUser}
                  className="bg-destructive text-destructive-foreground"
                >
                  {isDeleting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  {t('backOfficeUserManagement.deleteUser')}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>

          <AlertDialog open={isTerminateConfirmOpen} onOpenChange={setIsTerminateConfirmOpen}>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  {t('backOfficeUserManagement.terminateSession')}?
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {t('backOfficeUserManagement.terminateSessionDescription')}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel onClick={() => setSelectedUser(null)}>
                  {t('backOfficeUserManagement.cancel')}
                </AlertDialogCancel>
                <AlertDialogAction
                  onClick={handleTerminateSession}
                  className="bg-destructive text-destructive-foreground"
                >
                  {t('backOfficeUserManagement.terminateSession')}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>

          <AlertDialog open={isRestartConfirmOpen} onOpenChange={setIsRestartConfirmOpen}>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{t('backOfficeUserManagement.restartSession')}?</AlertDialogTitle>
                <AlertDialogDescription>
                  {t('backOfficeUserManagement.restartSessionDescription')}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel onClick={() => setSelectedUser(null)}>
                  {t('backOfficeUserManagement.cancel')}
                </AlertDialogCancel>
                <AlertDialogAction
                  onClick={handleRestartSession}
                  className="bg-accent text-accent-foreground hover:bg-accent/90"
                >
                  {t('backOfficeUserManagement.restartSession')}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </>
      }
    >
      <Card>
        <CardHeader>
          <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            {/* Search */}
            <div className="relative w-full lg:w-96">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder={t('backOfficeUserManagement.searchPlaceholder')}
                className="pl-9"
              />
            </div>

            {/* Filters */}
            <div className="flex flex-wrap gap-2">
              <Select value={roleFilter} onValueChange={setRoleFilter}>
                <SelectTrigger className="w-[160px]">
                  <SelectValue placeholder={t('backOfficeUserManagement.role')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('backOfficeUserManagement.allRoles')}</SelectItem>
                  <SelectItem value="admin">{t('backOfficeUserManagement.super_admin')}</SelectItem>
                  <SelectItem value="manager">
                    {t('backOfficeUserManagement.supervisor')}
                  </SelectItem>
                  <SelectItem value="agent">{t('backOfficeUserManagement.agent')}</SelectItem>
                </SelectContent>
              </Select>

              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-[140px]">
                  <SelectValue placeholder={t('backOfficeUserManagement.status')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('backOfficeUserManagement.allStatus')}</SelectItem>
                  <SelectItem value="active">{t('backOfficeUserManagement.active')}</SelectItem>
                  <SelectItem value="pending">{t('backOfficeUserManagement.pending')}</SelectItem>
                  <SelectItem value="suspended">
                    {t('backOfficeUserManagement.suspended')}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardHeader>

        <CardContent>
          {/* Stats */}
          <div className="mb-4 grid grid-cols-2 gap-4 lg:grid-cols-4">
            <div className="rounded-lg bg-muted p-4">
              <p className="text-sm text-muted-foreground">
                {t('backOfficeUserManagement.totalUsers')}
              </p>
              <p className="text-2xl font-bold">{totalUsersCount}</p>
            </div>
            <div className="rounded-lg bg-status-valid/10 p-4">
              <p className="text-sm text-muted-foreground">
                {t('backOfficeUserManagement.activeNow')}
              </p>
              <p className="text-2xl font-bold text-status-valid">{activeNowCount}</p>
            </div>
            <div className="rounded-lg bg-muted p-4">
              <p className="text-sm text-muted-foreground">
                {t('backOfficeUserManagement.supervisors')}
              </p>
              <p className="text-2xl font-bold">{supervisorsCount}</p>
            </div>
            <div className="rounded-lg bg-muted p-4">
              <p className="text-sm text-muted-foreground">
                {t('backOfficeUserManagement.organizations')}
              </p>
              <p className="text-2xl font-bold">{organizationsCount}</p>
            </div>
          </div>

          {/* Table */}
          <div className="rounded-lg border">
            <Table>
              <TableHeader>
                <TableRow className="bg-muted/50">
                  <TableHead>{t('backOfficeUserManagement.user')}</TableHead>
                  <TableHead>{t('backOfficeUserManagement.role')}</TableHead>
                  <TableHead>{t('backOfficeUserManagement.organization')}</TableHead>
                  <TableHead>{t('backOfficeUserManagement.status')}</TableHead>
                  <TableHead>{t('backOfficeUserManagement.lastActive')}</TableHead>
                  <TableHead>{t('backOfficeUserManagement.controlsToday')}</TableHead>
                  <TableHead className="w-[80px]">
                    {t('backOfficeUserManagement.actions')}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoadingUsers ? (
                  <TableRow>
                    <TableCell colSpan={7} className="h-24 text-center">
                      {t('backOfficeUserManagement.loadingUsers')}
                    </TableCell>
                  </TableRow>
                ) : filteredUsers.length > 0 ? (
                  filteredUsers.map((user: UserProfile) => (
                    <TableRow key={user.id} className="group">
                      <TableCell>
                        <div className="flex items-center gap-3">
                          <Avatar>
                            <AvatarFallback className="bg-primary text-primary-foreground">
                              {user.avatarInitials || user.name.substring(0, 2).toUpperCase()}
                            </AvatarFallback>
                          </Avatar>
                          <div>
                            <p className="font-medium">{user.name}</p>
                            <p className="text-sm text-muted-foreground">{user.email}</p>
                          </div>
                        </div>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Shield className="h-4 w-4 text-muted-foreground" />
                          {user.role}
                        </div>
                      </TableCell>
                      <TableCell>{user.organization}</TableCell>
                      <TableCell>
                        <StatusBadge
                          variant={
                            user.status === 'ACTIVE'
                              ? 'valid'
                              : user.status === 'PENDING_ACTIVATION'
                                ? 'pending'
                                : 'critical'
                          }
                          size="sm"
                        >
                          {user.status === 'ACTIVE'
                            ? t('backOfficeUserManagement.active')
                            : user.status === 'PENDING_ACTIVATION'
                              ? t('backOfficeUserManagement.pending')
                              : t('backOfficeUserManagement.suspended')}
                        </StatusBadge>
                      </TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {t('backOfficeUserManagement.today')}
                      </TableCell>
                      <TableCell>
                        <span className="font-medium">0</span>
                      </TableCell>
                      <TableCell>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="text-muted-foreground hover:text-foreground"
                            >
                              <MoreVertical className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuLabel>
                              {t('backOfficeUserManagement.actions')}
                            </DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              onClick={() => {
                                setSelectedUser(user);
                                setIsEditUserOpen(true);
                              }}
                            >
                              <Edit className="mr-2 h-4 w-4" />
                              {t('backOfficeUserManagement.editUser')}
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                              <Key className="mr-2 h-4 w-4" />
                              {t('backOfficeUserManagement.resetPassword')}
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                              <Shield className="mr-2 h-4 w-4" />
                              {t('backOfficeUserManagement.managePermissions')}
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() => {
                                setSelectedUser(user);
                                setIsTerminateConfirmOpen(true);
                              }}
                              className="text-status-warning"
                              disabled={user.role !== 'agent'}
                            >
                              <UserX className="mr-2 h-4 w-4" />
                              {t('backOfficeUserManagement.terminateSession')}
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() => {
                                setSelectedUser(user);
                                setIsRestartConfirmOpen(true);
                              }}
                              className="text-status-valid"
                              disabled={user.role !== 'agent'}
                            >
                              <RefreshCw className="mr-2 h-4 w-4" />
                              {t('backOfficeUserManagement.restartSession')}
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              disabled={
                                resendLoadingUserId === user.id ||
                                user.role !== 'agent' ||
                                !(
                                  user.status === 'PENDING_ACTIVATION' ||
                                  user.status === 'SUSPENDED'
                                )
                              }
                              onClick={() => handleResendActivationCode(user)}
                            >
                              {resendLoadingUserId === user.id ? (
                                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                              ) : (
                                <RefreshCw className="mr-2 h-4 w-4" />
                              )}
                              Resend activation code
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              onClick={() => toggleStatus(user)}
                              className={
                                user.isActive ? 'text-status-warning' : 'text-status-valid'
                              }
                            >
                              {user.isActive ? (
                                <>
                                  <UserX className="mr-2 h-4 w-4" />
                                  {t('backOfficeUserManagement.deactivate')}
                                </>
                              ) : (
                                <>
                                  <UserCheck className="mr-2 h-4 w-4" />
                                  {t('backOfficeUserManagement.activate')}
                                </>
                              )}
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onClick={() => {
                                setSelectedUser(user);
                                setIsDeleteConfirmOpen(true);
                              }}
                              className="text-destructive"
                            >
                              <Trash2 className="mr-2 h-4 w-4" />
                              {t('backOfficeUserManagement.deleteUser')}
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell colSpan={7} className="h-24 text-center text-muted-foreground">
                      {t('backOfficeUserManagement.noUsersFound')}
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>
    </BackOfficeLayout>
  );
}
