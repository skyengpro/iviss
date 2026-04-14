import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useLogControl } from '../useLogControl';
import { createQueryWrapper } from '@/test/queryWrapper';
import { toast } from '@/hooks/ui/use-toast';

// Mock dependencies
vi.mock('@/openapi-rq/queries/queries', () => ({
  useCreateControl: vi.fn(),
}));

import { useCreateControl } from '@/openapi-rq/queries/queries';

vi.mock('@/hooks/ui/use-toast', () => ({
  toast: vi.fn(),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

const mockUser = { id: 'agent-1', organizationId: 'org-1' };
const mockStatusResults = {
  overall_status: 'valid',
  insurance: { status: 'valid' },
  technical: { status: 'valid' },
  police: { status: 'valid' },
  customs: { status: 'valid' },
};

describe('useLogControl', () => {
  let mockMutateAsync: any;

  beforeEach(() => {
    vi.clearAllMocks();

    mockMutateAsync = vi.fn().mockResolvedValue({ id: 'control-1' });
    vi.mocked(useCreateControl).mockReturnValue({
      mutateAsync: mockMutateAsync,
      isPending: false,
    } as any);
  });

  it('success path: calls mutateAsync, shows success toast, sets controlLogged = true', async () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useLogControl(), { wrapper: Wrapper });

    let success: boolean | undefined;

    await act(async () => {
      success = await result.current.logControl(
        mockUser as any,
        'CE123AB',
        mockStatusResults as any,
        null
      );
    });

    expect(mockMutateAsync).toHaveBeenCalledWith({
      requestBody: {
        plate_number: 'CE123AB',
        agent_id: 'agent-1',
        organization_id: 'org-1',
        latitude: 48.8566,
        longitude: 2.3522,
        address: 'Highway A1, KM 42',
        identification_mode: 'manual',
        ocr_confidence: 1.0,
        results: {
          registration: 'valid',
          insurance: 'valid',
          technical_inspection: 'valid',
          wanted_status: 'valid',
          customs_status: 'valid',
        },
        notes: 'Logged via mobile app',
      },
    });

    expect(toast).toHaveBeenCalledWith({
      title: 'logControl.successTitle',
      description: 'logControl.successDescription',
    });

    expect(success).toBe(true);
    expect(result.current.controlLogged).toBe(true);
  });

  it('error path: shows error toast, returns false', async () => {
    mockMutateAsync.mockRejectedValue(new Error('API failure'));

    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useLogControl(), { wrapper: Wrapper });

    let success: boolean | undefined;

    await act(async () => {
      success = await result.current.logControl(
        mockUser as any,
        'LT999HN',
        mockStatusResults as any,
        null
      );
    });

    expect(toast).toHaveBeenCalledWith({
      title: 'logControl.errorTitle',
      description: 'logControl.errorDescription',
      variant: 'destructive',
    });

    expect(success).toBe(false);
    expect(result.current.controlLogged).toBe(false);
  });

  it('no-ops and returns undefined when required args are missing', async () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useLogControl(), { wrapper: Wrapper });

    let success: boolean | undefined;

    await act(async () => {
      success = await result.current.logControl(
        null as any,
        'CE123',
        mockStatusResults as any,
        null
      );
    });

    expect(mockMutateAsync).not.toHaveBeenCalled();
    expect(success).toBeUndefined();

    await act(async () => {
      success = await result.current.logControl(
        mockUser as any,
        '',
        mockStatusResults as any,
        null
      );
    });

    expect(mockMutateAsync).not.toHaveBeenCalled();
    expect(success).toBeUndefined();
  });

  it('setControlLogged can reset the flag', () => {
    const { Wrapper } = createQueryWrapper();
    const { result } = renderHook(() => useLogControl(), { wrapper: Wrapper });

    act(() => {
      result.current.setControlLogged(true);
    });

    expect(result.current.controlLogged).toBe(true);

    act(() => {
      result.current.setControlLogged(false);
    });

    expect(result.current.controlLogged).toBe(false);
  });
});
