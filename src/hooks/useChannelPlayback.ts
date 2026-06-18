import { useCallback, useEffect, useState } from 'react';
import { usePlayerStore } from '../stores/player-store';
import {
  playChannel as tauriPlayChannel,
  stopPlayback as tauriStopPlayback,
  isPlaying as checkIsPlaying,
  getChannelEpg,
  playEpisodeWithSeason,
} from '../lib/tauri';
import { isIOS } from '../lib/platform';
import { logger } from '../lib/logger';
import type { Channel, Playlist } from '../types';

export interface PlaylistEpisode {
  id: string;
  title: string;
  extension: string;
}

interface UseChannelPlaybackResult {
  currentChannel: Channel | null;
  isPlaying: boolean;
  currentProgram: string | null;
  nextProgram: string | null;
  /** URLs to stream on iOS (HTML5 video); empty on desktop */
  iosStreamUrls: string[];
  play: (channel: Channel) => Promise<{ type: 'series'; channel: Channel } | void>;
  stop: () => Promise<void>;
  playEpisode: (
    episodeId: string,
    extension: string,
    title: string,
    playlist: Playlist,
    remainingEpisodes?: PlaylistEpisode[]
  ) => Promise<void>;
}

export function useChannelPlayback(): UseChannelPlaybackResult {
  const currentChannel = usePlayerStore((s) => s.currentChannel);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const currentProgram = usePlayerStore((s) => s.currentProgram);
  const nextProgram = usePlayerStore((s) => s.nextProgram);
  const setCurrentChannel = usePlayerStore((s) => s.setCurrentChannel);
  const setIsPlaying = usePlayerStore((s) => s.setIsPlaying);
  const setCurrentProgram = usePlayerStore((s) => s.setCurrentProgram);
  const setNextProgram = usePlayerStore((s) => s.setNextProgram);

  const [iosStreamUrls, setIosStreamUrls] = useState<string[]>([]);

  // Poll MPV status to detect when the player is closed externally (desktop only)
  useEffect(() => {
    if (!isPlaying || isIOS()) return;

    const interval = setInterval(async () => {
      try {
        const playing = await checkIsPlaying();
        if (!playing) {
          setIsPlaying(false);
          setCurrentChannel(null);
          setCurrentProgram(null);
          setNextProgram(null);
        }
      } catch (err) {
        logger.error('Failed to check playback status:', err);
      }
    }, 3000);

    return () => clearInterval(interval);
  }, [isPlaying, setIsPlaying, setCurrentChannel, setCurrentProgram, setNextProgram]);

  // Update EPG periodically while playing
  useEffect(() => {
    if (!isPlaying || !currentChannel?.epg_id) return;

    const interval = setInterval(async () => {
      try {
        const [current, next] = await getChannelEpg(currentChannel.epg_id!);
        setCurrentProgram(current);
        setNextProgram(next);
      } catch (err) {
        logger.error('Failed to update EPG:', err);
      }
    }, 60000);

    return () => clearInterval(interval);
  }, [isPlaying, currentChannel, setCurrentProgram, setNextProgram]);

  const play = useCallback(
    async (channel: Channel): Promise<{ type: 'series'; channel: Channel } | void> => {
      if (channel.content_type === 'series') {
        return { type: 'series', channel };
      }

      // Toggle off if same channel
      if (currentChannel?.id === channel.id && isPlaying) {
        if (isIOS()) {
          setIsPlaying(false);
          setCurrentChannel(null);
          setIosStreamUrls([]);
          setCurrentProgram(null);
          setNextProgram(null);
        } else {
          await tauriStopPlayback();
          setIsPlaying(false);
          setCurrentProgram(null);
          setNextProgram(null);
        }
        return;
      }

      if (isIOS()) {
        // iOS: set state only — IOSVideoPlayer renders the <video> element
        setCurrentChannel(channel);
        setIsPlaying(true);
        setIosStreamUrls([channel.url]);
      } else {
        // Desktop: delegate to MPV via Tauri
        try {
          await tauriPlayChannel(channel);
          setCurrentChannel(channel);
          setIsPlaying(true);
        } catch (err) {
          logger.error('Failed to play channel:', err);
          throw err;
        }
      }

      // Fetch EPG if available (same on both platforms)
      if (channel.epg_id) {
        try {
          const [current, next] = await getChannelEpg(channel.epg_id);
          setCurrentProgram(current);
          setNextProgram(next);
        } catch (err) {
          logger.error('Failed to fetch EPG:', err);
          setCurrentProgram(null);
          setNextProgram(null);
        }
      } else {
        setCurrentProgram(null);
        setNextProgram(null);
      }
    },
    [currentChannel, isPlaying, setCurrentChannel, setIsPlaying, setCurrentProgram, setNextProgram]
  );

  const stop = useCallback(async () => {
    if (isIOS()) {
      setIsPlaying(false);
      setCurrentChannel(null);
      setIosStreamUrls([]);
      setCurrentProgram(null);
      setNextProgram(null);
      return;
    }
    try {
      await tauriStopPlayback();
      setIsPlaying(false);
      setCurrentProgram(null);
      setNextProgram(null);
    } catch (err) {
      logger.error('Failed to stop playback:', err);
      throw err;
    }
  }, [setIsPlaying, setCurrentChannel, setCurrentProgram, setNextProgram]);

  const playEpisode = useCallback(
    async (
      episodeId: string,
      extension: string,
      title: string,
      playlist: Playlist,
      remainingEpisodes?: PlaylistEpisode[]
    ) => {
      if (!playlist.url || !playlist.xtream_username || !playlist.xtream_password) {
        logger.error('Missing Xtream credentials');
        throw new Error('Missing Xtream credentials');
      }

      if (isIOS()) {
        // Build episode URLs on the frontend — same formula as Rust series_domain
        const base = playlist.url.replace(/\/$/, '');
        const makeUrl = (id: string, ext: string) =>
          `${base}/series/${playlist.xtream_username}/${playlist.xtream_password}/${id}.${ext}`;

        const urls =
          remainingEpisodes && remainingEpisodes.length > 0
            ? remainingEpisodes.map((ep) => makeUrl(ep.id, ep.extension))
            : [makeUrl(episodeId, extension)];

        const episodeChannel: Channel = {
          id: -1,
          playlist_id: playlist.id ?? 0,
          name: title,
          url: urls[0],
          content_type: 'series',
          is_favorite: false,
          sort_order: 0,
        };

        setCurrentChannel(episodeChannel);
        setIsPlaying(true);
        setIosStreamUrls(urls);
        return;
      }

      // Desktop path
      try {
        if (remainingEpisodes && remainingEpisodes.length > 0) {
          await playEpisodeWithSeason(
            playlist.url,
            playlist.xtream_username,
            playlist.xtream_password,
            remainingEpisodes
          );
          setIsPlaying(true);
        } else {
          const episodeUrl = `${playlist.url.replace(/\/$/, '')}/series/${playlist.xtream_username}/${playlist.xtream_password}/${episodeId}.${extension}`;
          const episodeChannel: Channel = {
            id: -1,
            playlist_id: playlist.id ?? 0,
            name: title,
            url: episodeUrl,
            content_type: 'series',
            is_favorite: false,
            sort_order: 0,
          };
          await tauriPlayChannel(episodeChannel);
          setCurrentChannel(episodeChannel);
          setIsPlaying(true);
        }
      } catch (err) {
        logger.error('Failed to play episode:', err);
        throw err;
      }
    },
    [setCurrentChannel, setIsPlaying]
  );

  return {
    currentChannel,
    isPlaying,
    currentProgram,
    nextProgram,
    iosStreamUrls,
    play,
    stop,
    playEpisode,
  };
}
