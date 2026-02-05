import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { InstallPrompt } from './InstallPrompt';

// Mock the Drawer components to bypass animations and portals
vi.mock('@/components/ui/drawer', () => ({
  Drawer: ({ children, open }: { children: React.ReactNode; open: boolean }) =>
    open ? <div data-testid="drawer">{children}</div> : null,
  DrawerContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DrawerHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DrawerTitle: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DrawerDescription: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DrawerFooter: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

describe('InstallPrompt', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  it('should not show the drawer initially', () => {
    render(<InstallPrompt />);
    expect(screen.queryByTestId('drawer')).not.toBeInTheDocument();
  });

  it('should show the drawer when beforeinstallprompt is dispatched', () => {
    render(<InstallPrompt />);

    const event = new Event('beforeinstallprompt');
    // @ts-expect-error - prompt is not standard on Event
    event.prompt = vi.fn();
    // @ts-expect-error - userChoice is not standard on Event
    event.userChoice = Promise.resolve({ outcome: 'accepted' });
    event.preventDefault = vi.fn();

    act(() => {
      globalThis.dispatchEvent(event);
    });

    expect(screen.getByTestId('drawer')).toBeInTheDocument();
    expect(screen.getByText('Get the app')).toBeInTheDocument();
    expect(event.preventDefault).toHaveBeenCalled();
  });

  it('should not show if recently dismissed', () => {
    const sevenDaysInMs = 7 * 24 * 60 * 60 * 1000;
    localStorage.setItem('pwa-prompt-dismissed-until', (Date.now() + sevenDaysInMs).toString());

    render(<InstallPrompt />);

    const event = new Event('beforeinstallprompt');
    act(() => {
      globalThis.dispatchEvent(event);
    });

    expect(screen.queryByTestId('drawer')).not.toBeInTheDocument();
  });

  it('should handle the install flow correctly', async () => {
    render(<InstallPrompt />);

    const event = new Event('beforeinstallprompt');
    const promptSpy = vi.fn();
    // @ts-expect-error - prompt is not standard on Event
    event.prompt = promptSpy;
    // @ts-expect-error - userChoice is not standard on Event
    event.userChoice = Promise.resolve({ outcome: 'accepted' });

    act(() => {
      globalThis.dispatchEvent(event);
    });

    const installButton = screen.getByText('Install App');
    await act(async () => {
      fireEvent.click(installButton);
    });

    expect(promptSpy).toHaveBeenCalled();
    expect(screen.queryByTestId('drawer')).not.toBeInTheDocument();
  });

  it('should store dismissal in localStorage when Later is clicked', () => {
    render(<InstallPrompt />);

    const event = new Event('beforeinstallprompt');
    // @ts-expect-error - prompt is not standard on Event
    event.prompt = vi.fn();
    // @ts-expect-error - userChoice is not standard on Event
    event.userChoice = Promise.resolve({ outcome: 'dismissed' });

    act(() => {
      globalThis.dispatchEvent(event);
    });

    const laterButton = screen.getByText('Later');
    act(() => {
      fireEvent.click(laterButton);
    });

    expect(localStorage.getItem('pwa-prompt-dismissed-until')).toBeTruthy();
    expect(screen.queryByTestId('drawer')).not.toBeInTheDocument();
  });
});
