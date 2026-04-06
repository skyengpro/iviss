import { render, screen } from '@testing-library/react';
import { MobileNavigation } from '../MobileNavigation';
import { BrowserRouter, MemoryRouter } from 'react-router-dom';
import { describe, it, expect, vi } from 'vitest';

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe('MobileNavigation', () => {
  it('renders correctly with all nav items', () => {
    render(
      <BrowserRouter>
        <MobileNavigation />
      </BrowserRouter>
    );

    expect(screen.getByText('mobileNav.home')).toBeInTheDocument();
    expect(screen.getByText('mobileNav.scan')).toBeInTheDocument();
    expect(screen.getByText('mobileNav.search')).toBeInTheDocument();
    expect(screen.getByText('mobileNav.history')).toBeInTheDocument();
    expect(screen.getByText('mobileNav.profile')).toBeInTheDocument();
  });

  it('highlights the active item based on current location', () => {
    render(
      <MemoryRouter initialEntries={['/mobile/scan']}>
        <MobileNavigation />
      </MemoryRouter>
    );

    const scanLink = screen.getByRole('link', { name: /mobileNav\.scan/i });
    expect(scanLink).toHaveClass('text-accent');

    const homeLink = screen.getByRole('link', { name: /mobileNav\.home/i });
    expect(homeLink).toHaveClass('text-muted-foreground');
  });

  it('highlights sub-routes accurately', () => {
    render(
      <MemoryRouter initialEntries={['/mobile/history/detail/123']}>
        <MobileNavigation />
      </MemoryRouter>
    );

    const historyLink = screen.getByRole('link', { name: /mobileNav\.history/i });
    expect(historyLink).toHaveClass('text-accent');
  });
});
