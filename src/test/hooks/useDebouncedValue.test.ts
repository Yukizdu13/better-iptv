import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';

describe('useDebouncedValue', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should return initial value immediately', () => {
    const { result } = renderHook(() => useDebouncedValue('hello', 300));
    expect(result.current).toBe('hello');
  });

  it('should not update value before delay', () => {
    const { result, rerender } = renderHook(({ value, delay }) => useDebouncedValue(value, delay), {
      initialProps: { value: 'hello', delay: 300 },
    });

    rerender({ value: 'world', delay: 300 });
    expect(result.current).toBe('hello');
  });

  it('should update value after delay', () => {
    const { result, rerender } = renderHook(({ value, delay }) => useDebouncedValue(value, delay), {
      initialProps: { value: 'hello', delay: 300 },
    });

    rerender({ value: 'world', delay: 300 });

    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(result.current).toBe('world');
  });

  it('should reset timer on rapid changes', () => {
    const { result, rerender } = renderHook(({ value, delay }) => useDebouncedValue(value, delay), {
      initialProps: { value: 'a', delay: 300 },
    });

    rerender({ value: 'ab', delay: 300 });
    act(() => {
      vi.advanceTimersByTime(100);
    });

    rerender({ value: 'abc', delay: 300 });
    act(() => {
      vi.advanceTimersByTime(100);
    });

    // Only 200ms since last change, should still be 'a'
    expect(result.current).toBe('a');

    act(() => {
      vi.advanceTimersByTime(200);
    });
    // Now 300ms since 'abc' was set
    expect(result.current).toBe('abc');
  });
});
