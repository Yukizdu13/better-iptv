import { useCallback, useState } from 'react';
import { logger } from '../lib/logger';
import { isAppError, parseError, getErrorSeverity, type AppError } from '../types/errors';

/**
 * Toast notification type for error display
 */
export interface Toast {
  id: string;
  message: string;
  severity: 'error' | 'warning' | 'info';
  duration?: number;
}

/**
 * Error handler hook result
 */
interface UseErrorHandlerResult {
  /** Show an error message to the user */
  showError: (error: unknown, context?: string) => void;
  /** Currently active toast notifications */
  toasts: Toast[];
  /** Dismiss a specific toast */
  dismissToast: (id: string) => void;
  /** Clear all toasts */
  clearToasts: () => void;
}

/**
 * Custom hook for consistent error handling across the application
 *
 * Provides unified error handling with:
 * - Automatic error parsing (AppError, Error, string)
 * - Logging to backend
 * - Toast notification management
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { showError } = useErrorHandler();
 *
 *   const handleClick = async () => {
 *     try {
 *       await someAsyncOperation();
 *     } catch (e) {
 *       showError(e, 'Failed to perform operation');
 *     }
 *   };
 * }
 * ```
 */
export function useErrorHandler(): UseErrorHandlerResult {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const dismissToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const clearToasts = useCallback(() => {
    setToasts([]);
  }, []);

  const showError = useCallback(
    (error: unknown, context?: string) => {
      // Parse the error to get a user-friendly message
      const message = parseError(error);

      // Determine severity based on error type
      const severity = isAppError(error) ? getErrorSeverity(error as AppError) : 'error';

      // Log the error with context
      const logMessage = context ? `${context}: ${message}` : message;

      if (severity === 'error') {
        logger.error(logMessage, error);
      } else if (severity === 'warning') {
        logger.warn(logMessage, error);
      } else {
        logger.info(logMessage, error);
      }

      // Create toast notification
      const toast: Toast = {
        id: `toast-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        message: context ? `${context}: ${message}` : message,
        severity,
        duration: severity === 'error' ? 8000 : 5000,
      };

      setToasts((prev) => [...prev, toast]);

      // Auto-dismiss after duration
      if (toast.duration) {
        setTimeout(() => {
          dismissToast(toast.id);
        }, toast.duration);
      }
    },
    [dismissToast]
  );

  return {
    showError,
    toasts,
    dismissToast,
    clearToasts,
  };
}

/**
 * Wrap an async function with error handling
 *
 * @example
 * ```tsx
 * const { showError } = useErrorHandler();
 *
 * const safeOperation = withErrorHandling(
 *   async () => await riskyOperation(),
 *   showError,
 *   'Operation failed'
 * );
 *
 * // Use in event handler
 * onClick={safeOperation}
 * ```
 */
export function withErrorHandling<T extends unknown[], R>(
  fn: (...args: T) => Promise<R>,
  onError: (error: unknown, context?: string) => void,
  context?: string
): (...args: T) => Promise<R | undefined> {
  return async (...args: T): Promise<R | undefined> => {
    try {
      return await fn(...args);
    } catch (e) {
      onError(e, context);
      return undefined;
    }
  };
}
