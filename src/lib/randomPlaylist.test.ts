import { describe, it, expect } from 'vitest';
import { buildBalancedPlaylist, shuffle } from './randomPlaylist';
import type { QueuedEpisode } from '../types';

function series(channelId: number, count: number): QueuedEpisode[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `${channelId}-${i}`,
    title: `S${channelId} E${i}`,
    extension: 'mkv',
    seriesChannelId: channelId,
    season: '1',
    episodeNum: i,
  }));
}

describe('buildBalancedPlaylist', () => {
  it('gives each series an equal share (700 vs 40 episodes)', () => {
    const queue = buildBalancedPlaylist([series(1, 700), series(2, 40)], 30);
    expect(queue).toHaveLength(30);
    expect(queue.filter((e) => e.seriesChannelId === 1)).toHaveLength(15);
    expect(queue.filter((e) => e.seriesChannelId === 2)).toHaveLength(15);
  });

  it('never queues the same episode twice', () => {
    const queue = buildBalancedPlaylist([series(1, 5), series(2, 50)], 30);
    expect(new Set(queue.map((e) => e.id)).size).toBe(queue.length);
  });

  it('does not chain two episodes of the same series when others remain', () => {
    const queue = buildBalancedPlaylist([series(1, 100), series(2, 100), series(3, 100)], 30);
    for (let i = 1; i < queue.length; i++) {
      expect(queue[i].seriesChannelId).not.toBe(queue[i - 1].seriesChannelId);
    }
  });

  it('fills the rest from larger series when a small one runs out', () => {
    const queue = buildBalancedPlaylist([series(1, 3), series(2, 100)], 30);
    expect(queue).toHaveLength(30);
    expect(queue.filter((e) => e.seriesChannelId === 1)).toHaveLength(3);
  });

  it('handles empty input', () => {
    expect(buildBalancedPlaylist([], 30)).toEqual([]);
    expect(buildBalancedPlaylist([[]], 30)).toEqual([]);
  });
});

describe('shuffle', () => {
  it('keeps all elements and does not mutate the input', () => {
    const input = [1, 2, 3, 4, 5];
    const out = shuffle(input);
    expect([...out].sort()).toEqual(input);
    expect(input).toEqual([1, 2, 3, 4, 5]);
  });
});
