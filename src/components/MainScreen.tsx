import { useEffect, useState, memo, useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { usePlayerStore } from '../stores/player-store';
import { playChannel, stopPlayback, isPlaying as checkIsPlaying, getChannelEpg, playEpisodeWithSeason } from '../lib/tauri';
import { Search, Play, Square, Star, Tv, Film, Clapperboard, Settings as SettingsIcon } from 'lucide-react';
import SeriesView from './SeriesView';
import SettingsModal from './Settings';
import type { Channel } from '../types';
import { logger } from '../lib/logger';
import { useResponsiveGrid, getGridClasses } from '../hooks/useResponsiveGrid';

export default function MainScreen() {
  const {
    channels,
    filteredChannels,
    liveChannels,
    vodChannels,
    seriesChannels,
    searchQuery,
    contentTypeFilter,
    currentChannel,
    currentPlaylist,
    isPlaying,
    currentProgram,
    nextProgram,
    channelEpgData,
    epgRefreshTrigger,
    setSearchQuery,
    setContentTypeFilter,
    setFilteredChannels,
    setCurrentChannel,
    setIsPlaying,
    setCurrentProgram,
    setNextProgram,
    setChannelEpg,
  } = usePlayerStore();

  const [selectedSeries, setSelectedSeries] = useState<Channel | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const parentRef = useRef<HTMLDivElement>(null);

  // Responsive grid configuration
  const { columns, cardHeight, estimatedRowHeight } = useResponsiveGrid();

  // Filter channels when search query or content type changes
  useEffect(() => {
    // Get pre-filtered list based on content type (instant tab switching)
    let baseList: Channel[];
    switch (contentTypeFilter) {
      case 'live':
        baseList = liveChannels;
        break;
      case 'vod':
        baseList = vodChannels;
        break;
      case 'series':
        baseList = seriesChannels;
        break;
      default:
        baseList = channels;
    }

    // Apply search filter on top of pre-filtered list
    if (searchQuery.trim() !== '') {
      const query = searchQuery.toLowerCase();
      const filtered = baseList.filter(
        (channel) =>
          channel.name.toLowerCase().includes(query) ||
          channel.group_name?.toLowerCase().includes(query)
      );
      setFilteredChannels(filtered);
    } else {
      setFilteredChannels(baseList);
    }
  }, [searchQuery, contentTypeFilter, channels, liveChannels, vodChannels, seriesChannels, setFilteredChannels]);

  // Poll MPV playback status to detect when player is closed externally
  useEffect(() => {
    if (!isPlaying) return;

    const interval = setInterval(async () => {
      try {
        const playing = await checkIsPlaying();
        if (!playing) {
          // MPV was closed externally, update UI
          setIsPlaying(false);
          setCurrentChannel(null);
          setCurrentProgram(null);
          setNextProgram(null);
        }
      } catch (err) {
        logger.error('Failed to check playback status:', err);
      }
    }, 1000); // Check every second

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
    }, 60000); // Update every minute

    return () => clearInterval(interval);
  }, [isPlaying, currentChannel, setCurrentProgram, setNextProgram]);

  // Fetch EPG for visible channels with epg_id (triggers on channel load or EPG refresh)
  useEffect(() => {
    const fetchVisibleChannelEpg = async () => {
      // Get unique channels with EPG IDs from filtered list
      const channelsWithEpg = filteredChannels.filter(
        (c) => c.epg_id && c.id && c.content_type === 'live'
      );

      // Fetch EPG for each channel (batched to avoid overwhelming the backend)
      const BATCH_SIZE = 10;
      for (let i = 0; i < channelsWithEpg.length; i += BATCH_SIZE) {
        const batch = channelsWithEpg.slice(i, i + BATCH_SIZE);
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
    };

    // Only fetch if we have channels
    if (filteredChannels.length > 0) {
      fetchVisibleChannelEpg();
    }
  }, [filteredChannels, setChannelEpg, epgRefreshTrigger]);

  // Refresh EPG periodically (every 5 minutes)
  useEffect(() => {
    const interval = setInterval(async () => {
      const channelsWithEpg = filteredChannels.filter(
        (c) => c.epg_id && c.id && c.content_type === 'live'
      );

      const BATCH_SIZE = 10;
      for (let i = 0; i < channelsWithEpg.length; i += BATCH_SIZE) {
        const batch = channelsWithEpg.slice(i, i + BATCH_SIZE);
        await Promise.all(
          batch.map(async (channel) => {
            try {
              const [current] = await getChannelEpg(channel.epg_id!);
              if (current && channel.id) {
                setChannelEpg(channel.id, current);
              }
            } catch (err) {
              logger.debug(`Failed to refresh EPG for ${channel.name}:`, err);
            }
          })
        );
      }
    }, 300000); // 5 minutes

    return () => clearInterval(interval);
  }, [filteredChannels, setChannelEpg]);

  // Virtual scrolling setup - virtualize by rows (dynamic items per row)
  const rowCount = Math.ceil(filteredChannels.length / columns);

  const rowVirtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => parentRef.current,
    estimateSize: () => estimatedRowHeight,
    overscan: 3, // Render 3 extra rows above/below viewport
  });

  const handlePlayChannel = async (channel: Channel) => {
    // If it's a series, open the series view instead of playing
    if (channel.content_type === 'series') {
      setSelectedSeries(channel);
      return;
    }

    try {
      if (currentChannel?.id === channel.id && isPlaying) {
        await stopPlayback();
        setIsPlaying(false);
        setCurrentProgram(null);
        setNextProgram(null);
      } else {
        await playChannel(channel);
        setCurrentChannel(channel);
        setIsPlaying(true);

        // Fetch EPG data if channel has EPG ID
        if (channel.epg_id) {
          try {
            const [current, next] = await getChannelEpg(channel.epg_id);
            setCurrentProgram(current);
            setNextProgram(next);
          } catch (err) {
            logger.error('Failed to fetch EPG:', err);
            // Don't fail the whole playback if EPG fails
            setCurrentProgram(null);
            setNextProgram(null);
          }
        } else {
          setCurrentProgram(null);
          setNextProgram(null);
        }
      }
    } catch (err) {
      logger.error('Failed to play channel:', err);
    }
  };

  const handlePlayEpisode = async (
    episodeId: string,
    extension: string,
    title: string,
    remainingEpisodes?: Array<{ id: string; title: string; extension: string }>
  ) => {
    if (!currentPlaylist?.url || !currentPlaylist.xtream_username || !currentPlaylist.xtream_password) {
      logger.error('Missing Xtream credentials');
      return;
    }

    try {
      // If we have remaining episodes (season playlist), play them all
      if (remainingEpisodes && remainingEpisodes.length > 0) {
        await playEpisodeWithSeason(
          currentPlaylist.url,
          currentPlaylist.xtream_username,
          currentPlaylist.xtream_password,
          remainingEpisodes
        );
        setIsPlaying(true);
      } else {
        // Fallback: play single episode (shouldn't happen with series, but kept for safety)
        const episodeUrl = `${currentPlaylist.url.replace(/\/$/, '')}/series/${currentPlaylist.xtream_username}/${currentPlaylist.xtream_password}/${episodeId}.${extension}`;

        const episodeChannel: Channel = {
          playlist_id: currentPlaylist.id || 0,
          name: title,
          url: episodeUrl,
          content_type: 'series',
          is_favorite: false,
          sort_order: 0,
        };

        await playChannel(episodeChannel);
        setCurrentChannel(episodeChannel);
        setIsPlaying(true);
      }
    } catch (err) {
      logger.error('Failed to play episode:', err);
    }
  };

  // If a series is selected, show the SeriesView
  if (selectedSeries && currentPlaylist?.url && currentPlaylist.xtream_username && currentPlaylist.xtream_password) {
    // Extract series ID from the URL (format: /series/user/pass/SERIES_ID.mp4)
    const urlParts = selectedSeries.url?.split('/');
    const seriesIdWithExt = urlParts?.[urlParts.length - 1];
    const seriesId = seriesIdWithExt ? parseInt(seriesIdWithExt.replace(/\.\w+$/, ''), 10) : NaN;

    // Handle invalid series ID
    if (isNaN(seriesId)) {
      logger.error('Failed to parse series ID from URL:', selectedSeries.url);
      return (
        <div className="h-screen flex items-center justify-center bg-gray-900">
          <div className="text-center">
            <p className="text-red-400 mb-4">Failed to load series: Invalid URL format</p>
            <button
              onClick={() => setSelectedSeries(null)}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
            >
              Go Back
            </button>
          </div>
        </div>
      );
    }

    return (
      <SeriesView
        seriesId={seriesId}
        seriesName={selectedSeries.name}
        serverUrl={currentPlaylist.url}
        username={currentPlaylist.xtream_username}
        password={currentPlaylist.xtream_password}
        onBack={() => setSelectedSeries(null)}
        onPlayEpisode={handlePlayEpisode}
      />
    );
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4">
        <div className="mx-auto px-2 flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Better IPTV
          </h1>
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {channels.length} channels
            </span>
            <button
              onClick={() => setShowSettings(true)}
              className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              title="Settings"
            >
              <SettingsIcon className="w-5 h-5 text-gray-600 dark:text-gray-400" />
            </button>
          </div>
        </div>
      </div>

      {/* Search Bar */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4">
        <div className="mx-auto px-2">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search channels..."
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
            />
          </div>
        </div>
      </div>

      {/* Content Type Tabs */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div className="mx-auto px-6">
          <div className="flex gap-2 overflow-x-auto" role="tablist" aria-label="Content type filter">
            <button
              role="tab"
              aria-selected={contentTypeFilter === 'all'}
              aria-controls="channel-list"
              onClick={() => setContentTypeFilter('all')}
              className={`flex items-center gap-2 px-4 py-3 font-medium transition-colors border-b-2 whitespace-nowrap ${
                contentTypeFilter === 'all'
                  ? 'text-blue-600 dark:text-blue-400 border-blue-600 dark:border-blue-400'
                  : 'text-gray-600 dark:text-gray-400 border-transparent hover:text-gray-900 dark:hover:text-gray-200'
              }`}
            >
              All
            </button>
            <button
              role="tab"
              aria-selected={contentTypeFilter === 'live'}
              aria-controls="channel-list"
              onClick={() => setContentTypeFilter('live')}
              className={`flex items-center gap-2 px-4 py-3 font-medium transition-colors border-b-2 whitespace-nowrap ${
                contentTypeFilter === 'live'
                  ? 'text-blue-600 dark:text-blue-400 border-blue-600 dark:border-blue-400'
                  : 'text-gray-600 dark:text-gray-400 border-transparent hover:text-gray-900 dark:hover:text-gray-200'
              }`}
            >
              <Tv className="w-4 h-4" aria-hidden="true" />
              Live TV
            </button>
            <button
              role="tab"
              aria-selected={contentTypeFilter === 'vod'}
              aria-controls="channel-list"
              onClick={() => setContentTypeFilter('vod')}
              className={`flex items-center gap-2 px-4 py-3 font-medium transition-colors border-b-2 whitespace-nowrap ${
                contentTypeFilter === 'vod'
                  ? 'text-blue-600 dark:text-blue-400 border-blue-600 dark:border-blue-400'
                  : 'text-gray-600 dark:text-gray-400 border-transparent hover:text-gray-900 dark:hover:text-gray-200'
              }`}
            >
              <Film className="w-4 h-4" aria-hidden="true" />
              Movies
            </button>
            <button
              role="tab"
              aria-selected={contentTypeFilter === 'series'}
              aria-controls="channel-list"
              onClick={() => setContentTypeFilter('series')}
              className={`flex items-center gap-2 px-4 py-3 font-medium transition-colors border-b-2 whitespace-nowrap ${
                contentTypeFilter === 'series'
                  ? 'text-blue-600 dark:text-blue-400 border-blue-600 dark:border-blue-400'
                  : 'text-gray-600 dark:text-gray-400 border-transparent hover:text-gray-900 dark:hover:text-gray-200'
              }`}
            >
              <Clapperboard className="w-4 h-4" aria-hidden="true" />
              Series
            </button>
          </div>
        </div>
      </div>

      {/* Channel List with Virtual Scrolling */}
      <div ref={parentRef} className="flex-1 overflow-y-auto" id="channel-list" role="tabpanel" aria-label="Channel list">
        <div className="mx-auto p-4">
          {filteredChannels.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-gray-500 dark:text-gray-400">
                {searchQuery ? 'No channels found' : 'No channels available'}
              </p>
            </div>
          ) : (
            <div
              style={{
                height: `${rowVirtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const startIndex = virtualRow.index * columns;
                const rowItems = filteredChannels.slice(startIndex, startIndex + columns);

                return (
                  <div
                    key={virtualRow.key}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: '100%',
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                  >
                    <div className={`grid ${getGridClasses(columns)} gap-4`}>
                      {rowItems.map((channel) => (
                        <ChannelCard
                          key={channel.id}
                          channel={channel}
                          isPlaying={currentChannel?.id === channel.id && isPlaying}
                          onPlay={() => handlePlayChannel(channel)}
                          currentProgram={channel.id ? channelEpgData.get(channel.id) : undefined}
                          cardHeight={cardHeight}
                        />
                      ))}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Now Playing Bar */}
      {currentChannel && (
        <div className="bg-blue-600 text-white p-4">
          <div className="mx-auto px-2 flex items-center justify-between">
            <div className="flex items-center gap-4">
              {currentChannel.logo && (
                <div className="w-12 h-12 bg-gray-900 rounded flex items-center justify-center p-1">
                  <img
                    src={currentChannel.logo}
                    alt={currentChannel.name}
                    className="max-w-full max-h-full object-contain"
                  />
                </div>
              )}
              <div>
                <p className="font-medium">{currentChannel.name}</p>
                <p className="text-sm text-blue-100">
                  {currentChannel.group_name || 'Live TV'}
                </p>
                {currentProgram && (
                  <p className="text-sm text-blue-200 mt-1">
                    <span className="font-medium">Now showing:</span> {currentProgram}
                  </p>
                )}
                {nextProgram && (
                  <p className="text-xs text-blue-200 mt-0.5">
                    <span className="font-medium">Next up:</span> {nextProgram}
                  </p>
                )}
              </div>
            </div>
            <button
              onClick={async () => {
                await stopPlayback();
                setIsPlaying(false);
              }}
              className="bg-white/20 hover:bg-white/30 p-2 rounded-lg transition-colors"
            >
              <Square className="w-5 h-5" />
            </button>
          </div>
        </div>
      )}

      {/* Settings Modal */}
      {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
    </div>
  );
}

interface ChannelCardProps {
  channel: {
    id?: number;
    name: string;
    logo?: string;
    group_name?: string;
    is_favorite: boolean;
    content_type: string;
    epg_id?: string;
  };
  isPlaying: boolean;
  onPlay: () => void;
  currentProgram?: string;
  cardHeight: number;
}

const ChannelCard = memo(function ChannelCard({ channel, isPlaying, onPlay, currentProgram, cardHeight }: ChannelCardProps) {
  // Calculate dynamic image height (approximately 45% of card height)
  const imageHeight = Math.max(80, Math.round(cardHeight * 0.45));

  // Scale text and padding based on card height
  const isLarge = cardHeight > 280;
  const isSmall = cardHeight < 220;

  return (
    <div
      className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden hover:shadow-md transition-shadow flex flex-col"
      style={{ height: `${cardHeight}px` }}
    >
      <div className="relative bg-gray-900 flex-shrink-0">
        {channel.logo ? (
          <div
            className="w-full bg-gray-900 flex items-center justify-center p-2"
            style={{ height: `${imageHeight}px` }}
          >
            <img
              src={channel.logo}
              alt={channel.name}
              loading="lazy"
              className="max-w-full max-h-full object-contain"
            />
          </div>
        ) : (
          <div
            className="w-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center"
            style={{ height: `${imageHeight}px` }}
          >
            <span className={`font-bold text-white ${isLarge ? 'text-4xl' : isSmall ? 'text-2xl' : 'text-3xl'}`}>
              {channel.name.charAt(0).toUpperCase()}
            </span>
          </div>
        )}
        {channel.is_favorite && (
          <div className="absolute top-2 right-2 bg-yellow-400 rounded-full p-1">
            <Star className={`${isSmall ? 'w-3 h-3' : 'w-4 h-4'} text-white fill-white`} />
          </div>
        )}
      </div>
      <div className={`${isLarge ? 'p-4' : isSmall ? 'p-2' : 'p-3'} flex flex-col flex-1 min-h-0`}>
        <h3 className={`font-medium text-gray-900 dark:text-white truncate ${isLarge ? 'text-base' : 'text-sm'}`}>
          {channel.name}
        </h3>
        {channel.group_name && (
          <p className={`text-gray-500 dark:text-gray-400 mt-0.5 truncate ${isSmall ? 'text-[10px]' : 'text-xs'}`}>
            {channel.group_name}
          </p>
        )}
        {currentProgram && channel.content_type === 'live' && (
          <p className={`text-blue-600 dark:text-blue-400 mt-0.5 truncate ${isSmall ? 'text-[10px]' : 'text-xs'}`} title={currentProgram}>
            📺 {currentProgram}
          </p>
        )}
        <div className="flex-1" />
        <button
          onClick={onPlay}
          className={`w-full rounded-md font-medium transition-colors flex items-center justify-center gap-2 ${
            isLarge ? 'py-2.5 px-4 mt-3' : isSmall ? 'py-1.5 px-3 mt-2 text-sm' : 'py-2 px-4 mt-2'
          } ${
            isPlaying
              ? 'bg-red-600 hover:bg-red-700 text-white'
              : channel.content_type === 'series'
              ? 'bg-purple-600 hover:bg-purple-700 text-white'
              : 'bg-blue-600 hover:bg-blue-700 text-white'
          }`}
        >
          {isPlaying ? (
            <>
              <Square className={`${isSmall ? 'w-3 h-3' : 'w-4 h-4'}`} />
              Stop
            </>
          ) : channel.content_type === 'series' ? (
            <>
              <Clapperboard className={`${isSmall ? 'w-3 h-3' : 'w-4 h-4'}`} />
              Browse
            </>
          ) : (
            <>
              <Play className={`${isSmall ? 'w-3 h-3' : 'w-4 h-4'}`} />
              Play
            </>
          )}
        </button>
      </div>
    </div>
  );
});
