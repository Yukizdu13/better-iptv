import { usePlayerStore } from '../stores/player-store';
import { mpvSeek } from './tauri';
import { logger } from './logger';

/**
 * Seek the MPV player by a relative amount and report the outcome in the Now Playing bar.
 *
 * The backend waits (a few seconds max) for MPV to finish opening the stream, so a seek
 * requested right after starting playback is applied instead of being silently dropped.
 */
export async function seekBy(seconds: number): Promise<void> {
  const { showPlaybackNotice } = usePlayerStore.getState();
  const label = seconds > 0 ? `+${seconds}s` : `${seconds}s`;
  try {
    await mpvSeek(seconds);
    showPlaybackNotice(label);
  } catch (err) {
    logger.error(`Failed to seek ${label}:`, err);
    showPlaybackNotice(`Seek ${label} impossible : ${String(err)}`, true);
  }
}
