import { useQuery, useMutation, useInfiniteQuery } from '@tanstack/react-query';
import * as options from '../generated/@tanstack/react-query.gen';

// ── Queries ──────────────────────────────────────────────────────────────────

export const useGetControls = (params?: any) => 
    useQuery(options.getControlsOptions(params));

export const useGetUserProfile = (params?: any) => 
    useQuery(options.getUserProfileOptions(params));

export const useGetDashboardStats = (params?: any) => 
    useQuery(options.getDashboardStatsOptions(params));

export const useListOrganizations = (params?: any) => 
    useQuery(options.listOrganizationsOptions(params));

export const useListUsers = (params?: any) => 
    useQuery(options.listUsersOptions(params));

export const useListPendingSubmissions = (params?: any) => 
    useQuery(options.listPendingSubmissionsOptions(params));

export const useGetPendingSubmission = (params: any) => 
    useQuery(options.getPendingSubmissionOptions(params));

export const useGetSubmissionAuditLog = (params: any) => 
    useQuery(options.getSubmissionAuditLogOptions(params));

export const useGetControlsPaged = (params?: any) => 
    useQuery(options.getControlsPagedOptions(params));

export const useGetUser = (params: any) => 
    useQuery(options.getUserOptions(params));

export const useHealth = (params?: any) => 
    useQuery(options.healthOptions(params));

export const useListAuditLogs = (params?: any) => 
    useQuery(options.listAuditLogsOptions(params));

export const useExportAuditLogs = (params?: any) => 
    useQuery(options.exportAuditLogsOptions(params));

export const useGetControlActivity = (params?: any) =>
    useQuery(options.getControlActivityOptions(params));

export const useGetRecentAlerts = (params?: any) =>
    useQuery(options.getRecentAlertsOptions(params));

export const useGetTopAgents = (params?: any) =>
    useQuery(options.getTopAgentsOptions(params));

export const useGetActivityFeed = (params?: any) =>
    useQuery(options.getActivityFeedOptions(params));

// ── Infinite Queries ─────────────────────────────────────────────────────────

export const useGetControlsPagedInfinite = (params?: any) =>
    useInfiniteQuery(options.getControlsPagedInfiniteOptions(params));

// ── Mutations ────────────────────────────────────────────────────────────────

export const useUpdateLocation = () => 
    useMutation(options.updateLocationMutation());

export const useSearchVehicleV1 = () => 
    useMutation(options.searchVehicleV1Mutation());

export const useSubmitVehicleV1 = () => 
    useMutation(options.submitVehicleV1Mutation());

export const useCreateControl = () => 
    useMutation(options.createControlMutation());

export const useProvisionUser = () => 
    useMutation(options.provisionUserMutation());

export const useDeleteUser = () => 
    useMutation(options.deleteUserMutation());

export const useUpdateUser = () => 
    useMutation(options.updateUserMutation());

export const useLoginUser = () => 
    useMutation(options.loginUserMutation());

export const useActivateDevice = () => 
    useMutation(options.activateDeviceMutation());

export const useRequestRefresh = () => 
    useMutation(options.requestRefreshMutation());

export const useVerifyRefresh = () => 
    useMutation(options.verifyRefreshMutation());

export const useRequestDailyLogin = () => 
    useMutation(options.requestDailyLoginMutation());

export const useVerifyDailyLogin = () => 
    useMutation(options.verifyDailyLoginMutation());

export const useTerminateSession = () => 
    useMutation(options.terminateSessionMutation());

export const useRestartSession = () => 
    useMutation(options.restartSessionMutation());

export const useResendActivationCode = () => 
    useMutation(options.resendActivationCodeMutation());
