import { render, screen, fireEvent } from '@testing-library/react';
import { MobileHeader } from '../MobileHeader';
import { BrowserRouter } from 'react-router-dom';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: vi.fn(),
    },
  }),
}));

// Mock useNavigate
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe('MobileHeader', () => {
  const mockOnMenuClick = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders correctly with default title', () => {
    render(
      <BrowserRouter>
        <MobileHeader onMenuClick={mockOnMenuClick} />
      </BrowserRouter>
    );

    expect(screen.getByText('IVISS')).toBeInTheDocument();
  });

  it('renders correctly with custom title', () => {
    render(
      <BrowserRouter>
        <MobileHeader onMenuClick={mockOnMenuClick} title="Custom Title" />
      </BrowserRouter>
    );

    expect(screen.getByText('Custom Title')).toBeInTheDocument();
  });

  it('calls onMenuClick when menu button is clicked', () => {
    render(
      <BrowserRouter>
        <MobileHeader onMenuClick={mockOnMenuClick} />
      </BrowserRouter>
    );

    const menuButton = screen.getAllByRole('button')[0];
    fireEvent.click(menuButton);

    expect(mockOnMenuClick).toHaveBeenCalledTimes(1);
  });

  it('navigates to profile when profile button is clicked', () => {
    render(
      <BrowserRouter>
        <MobileHeader onMenuClick={mockOnMenuClick} />
      </BrowserRouter>
    );

    const profileButton = screen.getAllByRole('button')[3]; // Menu, Globe, Bell, User
    fireEvent.click(profileButton);

    expect(mockNavigate).toHaveBeenCalledWith('/mobile/profile');
  });
});
