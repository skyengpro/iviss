import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import AdminLogin from '../AdminLogin';
import { BrowserRouter } from 'react-router-dom';
import { useAuth } from '@/hooks/auth/use-auth';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock useAuth
vi.mock('@/hooks/auth/use-auth', () => ({
  useAuth: vi.fn(),
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

describe('AdminLogin', () => {
  const mockLogin = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useAuth as any).mockReturnValue({
      login: mockLogin,
      isAuthenticated: false,
      user: null,
    });
  });

  it('renders correctly', () => {
    render(
      <BrowserRouter>
        <AdminLogin />
      </BrowserRouter>
    );

    expect(screen.getByText('Back-Office Administration')).toBeInTheDocument();
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
  });

  it('updates form fields correctly', () => {
    render(
      <BrowserRouter>
        <AdminLogin />
      </BrowserRouter>
    );

    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/password/i);

    fireEvent.change(emailInput, { target: { value: 'admin@iviss.gov' } });
    fireEvent.change(passwordInput, { target: { value: 'password123' } });

    expect(emailInput).toHaveValue('admin@iviss.gov');
    expect(passwordInput).toHaveValue('password123');
  });

  it('shows error if login fails', async () => {
    mockLogin.mockResolvedValue({ success: false, error: 'Invalid credentials' });

    render(
      <BrowserRouter>
        <AdminLogin />
      </BrowserRouter>
    );

    fireEvent.change(screen.getByLabelText(/email/i), { target: { value: 'admin@iviss.gov' } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: 'wrong' } });
    fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

    await waitFor(() => {
      expect(screen.getByText('Invalid credentials')).toBeInTheDocument();
    });
  });

  it('redirects to backoffice if already authenticated as admin', () => {
    (useAuth as any).mockReturnValue({
      login: mockLogin,
      isAuthenticated: true,
      user: { role: 'admin' },
    });

    render(
      <BrowserRouter>
        <AdminLogin />
      </BrowserRouter>
    );

    expect(mockNavigate).toHaveBeenCalledWith('/backoffice');
  });

  it('redirects to mobile if already authenticated as agent', () => {
    (useAuth as any).mockReturnValue({
      login: mockLogin,
      isAuthenticated: true,
      user: { role: 'agent' },
    });

    render(
      <BrowserRouter>
        <AdminLogin />
      </BrowserRouter>
    );

    expect(mockNavigate).toHaveBeenCalledWith('/mobile');
  });
});
