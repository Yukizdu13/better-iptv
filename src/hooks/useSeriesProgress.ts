import { useCallback, useEffect, useRef, useState } from 'react';
import { usePlayerStore } from '../stores/player-store';
import {
  getSetting,
  mpvGetPlaybackStatus,
  mpvPlaylistNext,
  mpvShowText,
  setLastEpisode,
} from '../lib/tauri';
import { isIOS } from '../lib/platform';
import { logger } from '../lib/logger';
import type { QueuedEpisode } from '../types';

/** Poll period of the MPV playback status */
const POLL_INTERVAL_MS = 1000;
/** An episode counts as "really watched" (and is saved as last episode) after this many seconds */
const WATCHED_THRESHOLD_SECONDS = 30;
/** Min delay between two saves of the playback position */
const SAVE_INTERVAL_MS = 5000;
/** The auto-next countdown is shown this many seconds before it triggers */
const COUNTDOWN_SECONDS = 10;

/** Global credits duration (seconds, 0 = disabled) */
export const CREDITS_SKIP_SETTING = 'credits_skip_seconds';
/** Per-series override of the credits duration (empty = use the global value) */
export function creditsSkipSeriesSetting(seriesChannelId: number): string {
  return `credits_skip_seconds_series_${seriesChannelId}`;
}

/** Credits duration to apply for a series: per-series override, else global value */
export async function getCreditsSkipSeconds(seriesChannelId?: number): Promise<number> {
  const parse = (v: string | null) => {
    const n = v != null && v.trim() !== '' ? Number(v) : NaN;
    return Number.isFinite(n) && n >= 0 ? n : null;
  };
  if (seriesChannelId != null) {
    const own = parse(await getSetting(creditsSkipSeriesSetting(seriesChannelId)));
    if (own != null) return own;
  }
  return parse(await getSetting(CREDITS_SKIP_SETTING)) ?? 0;
}

function saveProgress(ep: QueuedEpisode, position: number, duration: number | null) {
  if (ep.seriesChannelId == null || ep.season == null || ep.episodeNum == null) return;
  setLastEpisode(
    ep.seriesChannelId,
    ep.season,
    ep.id,
    ep.episodeNum,
    ep.title,
    position,
    duration
  ).catch((err) => logger.error('Failed to save episode progress:', err));
}

interface UseSeriesProgressResult {
  /** Seconds before auto-jumping to the next episode (null = no countdown) */
  autoNextIn: number | null;
  /** Watch the credits of the current episode (disable auto-next for it) */
  cancelAutoNext: () => void;
}

/**
 * Tracks series episodes played through MPV (desktop only):
 * - saves the last episode + playback position once really watched, then periodically
 *   (so a pause/stop can resume at the exact position)
 * - jumps to the next queued episode when the remaining time reaches the configured
 *   credits duration (global or per-series setting)
 */
export function useSeriesProgress(): UseSeriesProgressResult {
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const seriesSession = usePlayerStore((s) => s.seriesSession);
  const [autoNextIn, setAutoNextIn] = useState<number | null>(null);
  const cancelledRef = useRef(false);

  const cancelAutoNext = useCallback(() => {
    cancelledRef.current = true;
    setAutoNextIn(null);
  }, []);

  useEffect(() => {
    setAutoNextIn(null);
    if (!isPlaying || !seriesSession || seriesSession.length === 0 || isIOS()) return;

    let currentIdx: number | null = null;
    let creditsSkip = 0;
    let lastSaveAt = 0;
    let skippedIdx = -1;
    let busy = false;

    const tick = async () => {
      if (busy) return;
      busy = true;
      try {
        const status = await mpvGetPlaybackStatus();
        const idx = status.playlist_pos ?? 0;
        const ep = seriesSession[idx];
        if (!ep) return;

        if (idx !== currentIdx) {
          // New episode in MPV's playlist: reset per-episode state
          currentIdx = idx;
          lastSaveAt = 0;
          cancelledRef.current = false;
          setAutoNextIn(null);
          creditsSkip = await getCreditsSkipSeconds(ep.seriesChannelId).catch(() => 0);
          const { currentChannel, setCurrentChannel } = usePlayerStore.getState();
          if (currentChannel && currentChannel.name !== ep.title) {
            setCurrentChannel({ ...currentChannel, name: ep.title });
          }
        }

        const pos = status.time_pos;
        if (pos == null) return;

        if (pos >= WATCHED_THRESHOLD_SECONDS && Date.now() - lastSaveAt >= SAVE_INTERVAL_MS) {
          lastSaveAt = Date.now();
          saveProgress(ep, pos, status.duration);
        }

        // `c` pressed in the MPV window: keep the credits of this episode
        if (status.cancel_credits_pos === idx && !cancelledRef.current) {
          cancelledRef.current = true;
        }

        const hasNext = idx < seriesSession.length - 1;
        if (
          creditsSkip <= 0 ||
          !hasNext ||
          !status.duration ||
          cancelledRef.current ||
          status.paused
        ) {
          setAutoNextIn(null);
          return;
        }
        const remaining = status.duration - pos;
        if (remaining <= creditsSkip) {
          // Guard: MPV may still report the old playlist position for a moment after playlist-next
          if (skippedIdx !== idx) {
            skippedIdx = idx;
            saveProgress(ep, status.duration, status.duration);
            setAutoNextIn(null);
            await mpvPlaylistNext();
          }
        } else if (remaining <= creditsSkip + COUNTDOWN_SECONDS) {
          const secondsLeft = Math.ceil(remaining - creditsSkip);
          setAutoNextIn(secondsLeft);
          // The video is in MPV's own window: show the countdown there too
          mpvShowText(`Épisode suivant dans ${secondsLeft}s - C : garder le générique`, 1500).catch(
            () => undefined
          );
        } else {
          setAutoNextIn(null);
        }
      } catch (err) {
        // MPV still loading or just closed: the next tick will retry
        logger.debug('Series progress tick skipped:', err);
      } finally {
        busy = false;
      }
    };

    const interval = setInterval(tick, POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [isPlaying, seriesSession]);

  return { autoNextIn, cancelAutoNext };
}
