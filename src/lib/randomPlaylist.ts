import type { QueuedEpisode } from '../types';

/** Uniform in-place shuffle (Fisher-Yates) of a copy of the array */
export function shuffle<T>(items: readonly T[], rng: () => number = Math.random): T[] {
  const out = [...items];
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(rng() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

/**
 * Build a shuffled queue where every series gets a fair share, whatever its episode count.
 *
 * Shuffled round-robin: each round visits every series that still has episodes, in a fresh
 * random order, taking one random not-yet-queued episode from each. The first series of a
 * round is never the last one of the previous round (no back-to-back episodes of one series,
 * unless it is the only series left).
 */
export function buildBalancedPlaylist(
  episodesPerSeries: QueuedEpisode[][],
  size: number,
  rng: () => number = Math.random
): QueuedEpisode[] {
  const pools = episodesPerSeries.filter((eps) => eps.length > 0).map((eps) => shuffle(eps, rng));
  const queue: QueuedEpisode[] = [];
  let lastSeries: number | null = null;

  while (queue.length < size && pools.some((p) => p.length > 0)) {
    let order = shuffle(
      pools.filter((p) => p.length > 0),
      rng
    );
    if (order.length > 1 && order[0][0].seriesChannelId === lastSeries) {
      order = [...order.slice(1), order[0]];
    }
    for (const pool of order) {
      if (queue.length >= size) break;
      const ep = pool.pop()!;
      queue.push(ep);
      lastSeries = ep.seriesChannelId ?? null;
    }
  }
  return queue;
}
