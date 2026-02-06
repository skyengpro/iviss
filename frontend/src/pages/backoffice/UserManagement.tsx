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
} from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

import { useQuery } from '@tanstack/react-query';
import { mockAuthService, User } from '@/services/mockAuth';

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

  const { data: users = [], isLoading } = useQuery({
    queryKey: ['users'],
    queryFn: () => mockAuthService.getAllUsers(),
  });

  const filteredUsers = users.filter((user: User) => {
    const matchesSearch =
      user.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      user.email.toLowerCase().includes(searchQuery.toLowerCase()) ||
      user.organization.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesRole = roleFilter === 'all' || user.role === roleFilter;
    const matchesStatus =
      statusFilter === 'all' || (statusFilter === 'active' ? user.isActive : !user.isActive);

    return matchesSearch && matchesRole && matchesStatus;
  });

  return (
    <BackOfficeLayout
      title={t('backOfficeUserManagement.title')}
      subtitle={t('backOfficeUserManagement.subtitle')}
      actions={
        <Button className="gap-2 bg-accent text-accent-foreground hover:bg-accent/90">
          <Plus className="h-4 w-4" />
          {t('backOfficeUserManagement.addUser')}
        </Button>
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
                  <SelectItem value="super_admin">
                    {t('backOfficeUserManagement.super_admin')}
                  </SelectItem>
                  <SelectItem value="org_admin">
                    {t('backOfficeUserManagement.org_admin')}
                  </SelectItem>
                  <SelectItem value="supervisor">
                    {t('backOfficeUserManagement.supervisor')}
                  </SelectItem>
                  <SelectItem value="agent">{t('backOfficeUserManagement.agent')}</SelectItem>
                  <SelectItem value="operator">{t('backOfficeUserManagement.operator')}</SelectItem>
                </SelectContent>
              </Select>

              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-[140px]">
                  <SelectValue placeholder={t('backOfficeUserManagement.status')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('backOfficeUserManagement.allStatus')}</SelectItem>
                  <SelectItem value="active">{t('backOfficeUserManagement.active')}</SelectItem>
                  <SelectItem value="inactive">{t('backOfficeUserManagement.inactive')}</SelectItem>
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
              <p className="text-2xl font-bold">{users.length}</p>
            </div>
            <div className="rounded-lg bg-status-valid/10 p-4">
              <p className="text-sm text-muted-foreground">
                {t('backOfficeUserManagement.activeNow')}
              </p>
              <p className="text-2xl font-bold text-status-valid">89</p>
            </div>
            <div className="rounded-lg bg-muted p-4">
              <p className="text-sm text-muted-foreground">
                {t('backOfficeUserManagement.supervisors')}
              </p>
              <p className="text-2xl font-bold">12</p>
            </div>
            <div className="rounded-lg bg-muted p-4">
              <p className="text-sm text-muted-foreground">
                {t('backOfficeUserManagement.organizations')}
              </p>
              <p className="text-2xl font-bold">8</p>
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
                {isLoading ? (
                  <TableRow>
                    <TableCell colSpan={7} className="h-24 text-center">
                      {t('backOfficeUserManagement.loadingUsers')}
                    </TableCell>
                  </TableRow>
                ) : filteredUsers.length > 0 ? (
                  filteredUsers.map((user: User) => (
                    <TableRow key={user.id} className="group">
                      <TableCell>
                        <div className="flex items-center gap-3">
                          <Avatar>
                            <AvatarFallback className="bg-primary text-primary-foreground">
                              {user.avatarInitials}
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
                        <StatusBadge variant={user.isActive ? 'valid' : 'pending'} size="sm">
                          {user.isActive
                            ? t('backOfficeUserManagement.active')
                            : t('backOfficeUserManagement.inactive')}
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
                              className="opacity-0 group-hover:opacity-100"
                            >
                              <MoreVertical className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuLabel>
                              {t('backOfficeUserManagement.actions')}
                            </DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem>
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
                            <DropdownMenuSeparator />
                            {user.isActive ? (
                              <DropdownMenuItem className="text-status-warning">
                                <UserX className="mr-2 h-4 w-4" />
                                {t('backOfficeUserManagement.deactivate')}
                              </DropdownMenuItem>
                            ) : (
                              <DropdownMenuItem className="text-status-valid">
                                <UserCheck className="mr-2 h-4 w-4" />
                                {t('backOfficeUserManagement.activate')}
                              </DropdownMenuItem>
                            )}
                            <DropdownMenuItem className="text-destructive">
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
