import { useEffect, useRef } from 'react';
import { usePlayerStore } from '../stores/player-store';
import { getChannelEpg } from '../lib/tauri';
import { logger } from '../lib/logger';
import type { Channel } from '../types';

/**
 * Configuration for EPG fetching
 */
const EPG_CONFIG = {
  /** Number of channels to fetch in each batch */
  BATCH_SIZE: 10,
  /** Interval between full EPG refreshes (ms) */
  REFRESH_INTERVAL: 300000, // 5 minutes
};

/**
 * Hook result for EPG data
 */
interface UseEpgDataResult {
  /** EPG data map (channel ID -> current program title) */
  channelEpgData: Map<number, string>;
  /** Trigger a manual refresh of EPG data */
  refreshEpg: () => void;
}

/**
 * Custom hook for managing EPG (Electronic Program Guide) data
 *
 * Consolidates all EPG-related logic:
 * - Fetches EPG for visible live channels
 * - Batches requests to avoid overwhelming the backend
 * - Periodic refresh every 5 minutes
 * - Responds to external refresh triggers
 */
export function useEpgData(channels: Channel[]): UseEpgDataResult {
  const { channelEpgData, setChannelEpg, epgRefreshTrigger, triggerEpgRefresh } = usePlayerStore();

  // Track if we're currently fetching to avoid duplicate requests
  const isFetchingRef = useRef(false);

  // Fetch EPG for visible channels with EPG IDs
  useEffect(() => {
    const fetchVisibleChannelEpg = async () => {
      if (isFetchingRef.current) return;

      // Get unique live channels with EPG IDs
      const channelsWithEpg = channels.filter(
        (c) => c.epg_id && c.id && c.content_type === 'live'
      );

      if (channelsWithEpg.length === 0) return;

      isFetchingRef.current = true;

      try {
        // Fetch EPG in batches
        for (let i = 0; i < channelsWithEpg.length; i += EPG_CONFIG.BATCH_SIZE) {
          const batch = channelsWithEpg.slice(i, i + EPG_CONFIG.BATCH_SIZE);

          await Promise.all(
            batch.map(async (channel) => {
              try {
                const [current] = await getChannelEpg(channel.epg_id!);
                if (current && channel.id) {
                  setChannelEpg(channel.id, current);
                }
              } catch (err) {
                // Silently fail for individual channels
                logger.debug(`Failed to fetch EPG for ${channel.name}:`, err);
              }
            })
          );
        }
      } finally {
        isFetchingRef.current = false;
      }
    };

    // Only fetch if we have channels
    if (channels.length > 0) {
      fetchVisibleChannelEpg();
    }
  }, [channels, setChannelEpg, epgRefreshTrigger]);

  // Periodic EPG refresh
  useEffect(() => {
    const interval = setInterval(() => {
      triggerEpgRefresh();
    }, EPG_CONFIG.REFRESH_INTERVAL);

    return () => clearInterval(interval);
  }, [triggerEpgRefresh]);

  return {
    channelEpgData,
    refreshEpg: triggerEpgRefresh,
  };
}

/**
 * Hook for EPG data for a specific channel
 * Useful when you only need EPG for the current channel
 */
export function useChannelEpg(channelId: number | undefined): string | undefined {
  const { channelEpgData } = usePlayerStore();

  if (!channelId) return undefined;
  return channelEpgData.get(channelId);
}
