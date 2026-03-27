import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import { ErrorBoundary } from '../shared/ErrorBoundary';

// Mock the ErrorFallback component to avoid deep dependency issues
vi.mock('../shared/error/ErrorFallback', () => ({
  ErrorFallback: ({ error }: { error: Error | null; onReset: () => void }) => (
    <div data-testid="error-fallback">Error: {error?.message ?? 'Unknown'}</div>
  ),
}));

// A helper component that throws on demand
function ThrowingComponent({ shouldThrow }: { shouldThrow: boolean }) {
  if (shouldThrow) {
    throw new Error('Test render error');
  }
  return <div data-testid="child">All good</div>;
}

describe('ErrorBoundary', () => {
  beforeEach(() => {
    // Suppress React error boundary console.error noise in tests
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  it('should render children when no error occurs', () => {
    render(
      <ErrorBoundary>
        <ThrowingComponent shouldThrow={false} />
      </ErrorBoundary>
    );

    expect(screen.getByTestId('child')).toHaveTextContent('All good');
  });

  it('should render the default ErrorFallback when a child throws', () => {
    render(
      <ErrorBoundary>
        <ThrowingComponent shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByTestId('error-fallback')).toHaveTextContent('Error: Test render error');
  });

  it('should render custom fallback when provided', () => {
    const customFallback = <div data-testid="custom-fallback">Oops!</div>;

    render(
      <ErrorBoundary fallback={customFallback}>
        <ThrowingComponent shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByTestId('custom-fallback')).toHaveTextContent('Oops!');
  });
});
