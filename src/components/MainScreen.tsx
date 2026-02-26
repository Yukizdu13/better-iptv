import { useEffect, useState, useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { usePlayerStore } from '../stores/player-store';
import {
  playChannel,
  stopPlayback,
  isPlaying as checkIsPlaying,
  getChannelEpg,
  playEpisodeWithSeason,
  getChannelGroups,
  getStalePlaylistIds,
  getChannels,
} from '../lib/tauri';
import { CategoryBar } from './CategoryBar';
import { ChannelCard } from './ChannelCard';
import { SearchBar } from './SearchBar';
import { ContentTypeTabs } from './ContentTypeTabs';
import { NowPlayingBar } from './NowPlayingBar';
import { Settings as SettingsIcon } from 'lucide-react';
import SeriesView from './SeriesView';
import SettingsModal from './Settings';
import PinEntryModal from './modals/PinEntryModal';
import ConfirmationModal from './modals/ConfirmationModal';
import RefreshModal from './modals/RefreshModal';
import type { Channel } from '../types';
import { logger } from '../lib/logger';
import { useResponsiveGrid, getGridClasses } from '../hooks/useResponsiveGrid';
import { useEpgData } from '../hooks/useEpgData';
import { useKeyboardShortcuts } from '../hooks/useKeyboardShortcuts';
import { shouldBlockChannel } from '../lib/parentalControls';

export default function MainScreen() {
  const {
    channels,
    filteredChannels,
    liveChannels,
    vodChannels,
    seriesChannels,
    favoriteChannels,
    searchQuery,
    contentTypeFilter,
    categoryFilter,
    currentChannel,
    currentPlaylist,
    isPlaying,
    currentProgram,
    nextProgram,
    setSearchQuery,
    setContentTypeFilter,
    setFilteredChannels,
    setCurrentChannel,
    setIsPlaying,
    setCurrentProgram,
    setNextProgram,
    setCategories,
    parentalEnabled,
    parentalUnlocked,
    blockedChannelIds,
    blockedCategories,
    parentalAutoDetect,
    parentalVisibility,
    loadParentalSettings,
    setChannels,
    toggleChannelFavorite,
  } = usePlayerStore();

  // Use consolidated EPG hook for channel EPG data (with debouncing and caching)
  const { channelEpgData } = useEpgData(filteredChannels);

  const [selectedSeries, setSelectedSeries] = useState<Channel | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showPinModal, setShowPinModal] = useState(false);
  const [pendingChannel, setPendingChannel] = useState<Channel | null>(null);
  const parentRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<globalThis.HTMLInputElement>(null);
  const [showStalePrompt, setShowStalePrompt] = useState(false);
  const [stalePlaylistId, setStalePlaylistId] = useState<number | null>(null);
  const [showRefreshModal, setShowRefreshModal] = useState(false);

  // Global keyboard shortcuts (Space=play/stop, /=focus search, Escape=stop)
  useKeyboardShortcuts(searchInputRef);

  // Responsive grid configuration
  const { columns, cardHeight, estimatedRowHeight } = useResponsiveGrid();

  // Load parental settings on mount
  useEffect(() => {
    loadParentalSettings();
  }, [loadParentalSettings]);

  // Check for stale playlists on mount
  useEffect(() => {
    if (!currentPlaylist?.id) return;

    getStalePlaylistIds()
      .then((ids) => {
        if (ids.includes(currentPlaylist.id!)) {
          setStalePlaylistId(currentPlaylist.id!);
          setShowStalePrompt(true);
        }
      })
      .catch((err) => logger.error('Failed to check stale playlists:', err));
  }, [currentPlaylist?.id]);

  // Fetch categories when playlist or content type changes
  useEffect(() => {
    if (!currentPlaylist?.id) {
      setCategories([]);
      return;
    }

    if (contentTypeFilter === 'favorites') {
      setCategories([]);
      return;
    }

    const contentType = contentTypeFilter === 'all' ? undefined : contentTypeFilter;
    getChannelGroups(currentPlaylist.id, contentType)
      .then(setCategories)
      .catch((err) => {
        logger.error('Failed to fetch categories:', err);
        setCategories([]);
      });
  }, [currentPlaylist?.id, contentTypeFilter, setCategories]);

  // Filter channels when search query, content type, or category changes
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
      case 'favorites':
        baseList = favoriteChannels;
        break;
      default:
        baseList = channels;
    }

    // Apply category filter
    if (categoryFilter) {
      baseList = baseList.filter((channel) => channel.group_name === categoryFilter);
    }

    // Apply parental controls filter - only hide if visibility mode is 'hide'
    if (parentalEnabled && !parentalUnlocked && parentalVisibility === 'hide') {
      baseList = baseList.filter((channel) => {
        const blocked = shouldBlockChannel(channel, {
          enabled: parentalEnabled,
          autoDetect: parentalAutoDetect,
          blockedIds: blockedChannelIds,
          blockedCategories: blockedCategories,
          unlocked: parentalUnlocked,
        });
        return !blocked;
      });
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
  }, [
    searchQuery,
    contentTypeFilter,
    categoryFilter,
    channels,
    liveChannels,
    vodChannels,
    seriesChannels,
    favoriteChannels,
    setFilteredChannels,
    parentalEnabled,
    parentalUnlocked,
    parentalAutoDetect,
    blockedChannelIds,
    blockedCategories,
    parentalVisibility,
  ]);

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
    }, 2000); // Check every 2 seconds (reduced from 1s for performance)

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

    // Check if channel is blocked by parental controls
    const isBlocked = shouldBlockChannel(channel, {
      enabled: parentalEnabled,
      autoDetect: parentalAutoDetect,
      blockedIds: blockedChannelIds,
      blockedCategories: blockedCategories,
      unlocked: parentalUnlocked,
    });

    // If blocked and not unlocked, request PIN before playing
    if (isBlocked && parentalEnabled && !parentalUnlocked) {
      setPendingChannel(channel);
      setShowPinModal(true);
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
    if (
      !currentPlaylist?.url ||
      !currentPlaylist.xtream_username ||
      !currentPlaylist.xtream_password
    ) {
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

        // Create a temporary channel object for playback
        // Using id: -1 to indicate this is a virtual/temporary channel
        const episodeChannel: Channel = {
          id: -1,
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

  const handlePinSuccess = () => {
    setShowPinModal(false);
    // After PIN is verified, play the pending channel
    if (pendingChannel) {
      // Play the channel directly (PIN has been verified)
      playChannel(pendingChannel)
        .then(() => {
          setCurrentChannel(pendingChannel);
          setIsPlaying(true);
          setPendingChannel(null);
        })
        .catch((err) => {
          logger.error('Failed to play channel after PIN:', err);
        });
    }
  };

  // If a series is selected, show the SeriesView
  if (
    selectedSeries &&
    currentPlaylist?.url &&
    currentPlaylist.xtream_username &&
    currentPlaylist.xtream_password
  ) {
    // Extract series ID from the URL (format: /series/user/pass/SERIES_ID.mp4)
    const urlParts = selectedSeries.url?.split('/');
    const seriesIdWithExt = urlParts?.[urlParts.length - 1];
    const seriesId = seriesIdWithExt ? parseInt(seriesIdWithExt.replace(/\.\w+$/, ''), 10) : NaN;

    // Handle invalid series ID
    if (isNaN(seriesId)) {
      logger.error('Failed to parse series ID from URL:', selectedSeries.url);
      return (
        <div className="flex h-screen items-center justify-center bg-gray-900">
          <div className="text-center">
            <p className="mb-4 text-red-400">Failed to load series: Invalid URL format</p>
            <button
              onClick={() => setSelectedSeries(null)}
              className="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
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
    <div className="flex h-screen flex-col bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="border-b border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800">
        <div className="mx-auto flex items-center justify-between px-2">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Better IPTV</h1>
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {channels.length} channels
            </span>
            <button
              onClick={() => setShowSettings(true)}
              className="rounded-lg p-2 transition-colors hover:bg-gray-100 dark:hover:bg-gray-700"
              title="Settings"
            >
              <SettingsIcon className="h-5 w-5 text-gray-600 dark:text-gray-400" />
            </button>
          </div>
        </div>
      </div>

      {/* Search Bar */}
      <SearchBar ref={searchInputRef} value={searchQuery} onChange={setSearchQuery} />

      {/* Content Type Tabs */}
      <ContentTypeTabs activeFilter={contentTypeFilter} onFilterChange={setContentTypeFilter} />

      {/* Category Bar - horizontal scrollable chips */}
      <CategoryBar />

      {/* Channel List with Virtual Scrolling */}
      <div
        ref={parentRef}
        className="flex-1 overflow-y-auto"
        id="channel-list"
        role="tabpanel"
        aria-label="Channel list"
      >
        <div className="mx-auto p-4">
          {filteredChannels.length === 0 ? (
            <div className="py-12 text-center">
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
                      {rowItems.map((channel) => {
                        const isChannelBlocked = shouldBlockChannel(channel, {
                          enabled: parentalEnabled,
                          autoDetect: parentalAutoDetect,
                          blockedIds: blockedChannelIds,
                          blockedCategories: blockedCategories,
                          unlocked: parentalUnlocked,
                        });

                        return (
                          <ChannelCard
                            key={channel.id}
                            channel={channel}
                            isPlaying={currentChannel?.id === channel.id && isPlaying}
                            onPlay={() => handlePlayChannel(channel)}
                            onToggleFavorite={() => channel.id && toggleChannelFavorite(channel.id)}
                            currentProgram={channel.id ? channelEpgData.get(channel.id) : undefined}
                            cardHeight={cardHeight}
                            isBlocked={isChannelBlocked}
                            parentalVisibility={parentalVisibility}
                          />
                        );
                      })}
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
        <NowPlayingBar
          channel={currentChannel}
          currentProgram={currentProgram}
          nextProgram={nextProgram}
          onStop={async () => {
            await stopPlayback();
            setIsPlaying(false);
          }}
        />
      )}

      {/* Settings Modal */}
      {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}

      {/* PIN Entry Modal for blocked channels */}
      <PinEntryModal
        isOpen={showPinModal}
        onClose={() => {
          setShowPinModal(false);
          setPendingChannel(null);
        }}
        onSuccess={handlePinSuccess}
        mode="verify"
        title="Enter PIN to access this channel"
      />

      {/* Stale playlist prompt */}
      <ConfirmationModal
        isOpen={showStalePrompt}
        onClose={() => setShowStalePrompt(false)}
        onConfirm={() => {
          setShowStalePrompt(false);
          setShowRefreshModal(true);
        }}
        title="Playlist Update Available"
        message="Your playlist hasn't been updated in over 7 days. Would you like to refresh it now?"
        confirmText="Refresh Now"
        cancelText="Later"
      />

      {/* Refresh modal */}
      {stalePlaylistId && currentPlaylist && (
        <RefreshModal
          isOpen={showRefreshModal}
          onClose={() => {
            setShowRefreshModal(false);
            setStalePlaylistId(null);
          }}
          playlistId={stalePlaylistId}
          playlistName={currentPlaylist.name}
          onRefreshComplete={async () => {
            if (currentPlaylist.id) {
              try {
                const freshChannels = await getChannels(currentPlaylist.id);
                setChannels(freshChannels);
              } catch (err) {
                logger.error('Failed to reload channels after refresh:', err);
              }
            }
          }}
        />
      )}
    </div>
  );
}
