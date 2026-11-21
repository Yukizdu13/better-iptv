import { describe, it, expect, vi } from 'vitest';
import { loadAddedProfiles } from './accountLoad';

const profiles = [
  { id: 1, name: 'a' },
  { id: 2, name: 'b' },
  { id: 3, name: 'c' },
];

describe('loadAddedProfiles', () => {
  it('loads the active profile first, then the others, one at a time', async () => {
    const order: number[] = [];
    const refresh = vi.fn(async (id: number) => {
      order.push(id);
    });
    const steps: string[] = [];
    const failures = await loadAddedProfiles(
      profiles,
      2,
      refresh,
      (s) => steps.push(`${s.index}/${s.total}:${s.name}`),
      String
    );
    expect(order).toEqual([2, 1, 3]);
    expect(steps).toEqual(['1/3:b', '2/3:a', '3/3:c']);
    expect(failures).toEqual([]);
  });

  it('keeps going after a failure and reports it', async () => {
    const refresh = vi.fn(async (id: number) => {
      if (id === 1) throw { code: 'Http', details: 'indisponible' };
    });
    const failures = await loadAddedProfiles(
      profiles,
      null,
      refresh,
      () => {},
      (e) => String((e as { details: string }).details)
    );
    expect(refresh).toHaveBeenCalledTimes(3);
    expect(failures).toEqual([{ profile: profiles[0], message: 'indisponible' }]);
  });

  it('does nothing without profiles', async () => {
    const refresh = vi.fn();
    expect(await loadAddedProfiles([], null, refresh, () => {}, String)).toEqual([]);
    expect(refresh).not.toHaveBeenCalled();
  });
});
