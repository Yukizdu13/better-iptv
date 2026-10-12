import { useEffect, useState, useRef, useCallback, useMemo } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { usePlayerStore, EMPTY_ADVANCED_FILTERS } from '../stores/player-store';
import { getChannelGroups, getStalePlaylistIds, getChannels, getFavorites, getSetting } from '../lib/tauri';
import { isIOS } from '../lib/platform';
import { CategoryBar } from './CategoryBar';
import { ChannelCard } from './ChannelCard';
import { SearchBar } from './SearchBar';
import { ContentTypeTabs } from './ContentTypeTabs';
import { NowPlayingBar } from './NowPlayingBar';
import { IOSVideoPlayer } from './IOSVideoPlayer';
import { Settings as SettingsIcon, SlidersHorizontal, Shuffle } from 'lucide-react';
import SeriesView from './SeriesView';
import { FilterPanel } from './FilterPanel';
import VodView from './VodView';
import SettingsModal from './Settings';
import PinEntryModal from './modals/PinEntryModal';
import ConfirmationModal from './modals/ConfirmationModal';
import RefreshModal from './modals/RefreshModal';
import RandomPlayModal from './modals/RandomPlayModal';
import type { AdvancedFilters, Channel, QueuedEpisode } from '../types';
import { logger } from '../lib/logger';
import { useResponsiveGrid, getGridClasses } from '../hooks/useResponsiveGrid';
import { useEpgData } from '../hooks/useEpgData';
import { useKeyboardShortcuts } from '../hooks/useKeyboardShortcuts';
import { useSeriesProgress } from '../hooks/useSeriesProgress';
import { DEFAULT_FILTERS_SETTING } from './FilterPanel';
import { useChannelPlayback } from '../hooks/useChannelPlayback';
import { shouldBlockChannel } from '../lib/parentalControls';
import { useDebouncedValue } from '../hooks/useDebouncedValue';
import { useChannelFilter } from '../hooks/useChannelFilter';

export default function MainScreen() {
  // Channel data
  const channels = usePlayerStore((s) => s.channels);

  // Search & filters
  const searchQuery = usePlayerStore((s) => s.searchQuery);
  const contentTypeFilter = usePlayerStore((s) => s.contentTypeFilter);
  const categoryFilter = usePlayerStore((s) => s.categoryFilter);
  const setSearchQuery = usePlayerStore((s) => s.setSearchQuery);
  const setContentTypeFilter = usePlayerStore((s) => s.setContentTypeFilter);
  const setCategories = usePlayerStore((s) => s.setCategories);
  const debouncedSearchQuery = useDebouncedValue(searchQuery, 300);

  // Consolidated channel filtering (content type, category, parental, search)
  const filteredChannels = useChannelFilter(debouncedSearchQuery);

  // Playback (hook handles polling + EPG updates)
  const {
    currentChannel,
    isPlaying,
    currentProgram,
    nextProgram,
    iosStreamUrls,
    play: playChannelAction,
    stop: stopPlaybackAction,
    playEpisode: playEpisodeAction,
  } = useChannelPlayback();
  const currentPlaylist = usePlayerStore((s) => s.currentPlaylist);
  const playlists = usePlayerStore((s) => s.playlists);
  const setChannels = usePlayerStore((s) => s.setChannels);
  const toggleChannelFavorite = usePlayerStore((s) => s.toggleChannelFavorite);
  const otherProfileFavorites = usePlayerStore((s) => s.otherProfileFavorites);
  const setOtherProfileFavorites = usePlayerStore((s) => s.setOtherProfileFavorites);
  const seriesChannels = usePlayerStore((s) => s.seriesChannels);
  const [showRandomPlayModal, setShowRandomPlayModal] = useState(false);

  // Parental
  const parentalEnabled = usePlayerStore((s) => s.parentalEnabled);
  const parentalUnlocked = usePlayerStore((s) => s.parentalUnlocked);
  const blockedChannelIds = usePlayerStore((s) => s.blockedChannelIds);
  const blockedCategories = usePlayerStore((s) => s.blockedCategories);
  const parentalAutoDetect = usePlayerStore((s) => s.parentalAutoDetect);
  const parentalVisibility = usePlayerStore((s) => s.parentalVisibility);
  const loadParentalSettings = usePlayerStore((s) => s.loadParentalSettings);

  // Use consolidated EPG hook for channel EPG data (with debouncing and caching)
  const { channelEpgData } = useEpgData(filteredChannels);

  const [selectedSeries, setSelectedSeries] = useState<Channel | null>(null);
  const [selectedVod, setSelectedVod] = useState<Channel | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showFilters, setShowFilters] = useState(false);
  const advancedFilters = usePlayerStore((s) => s.advancedFilters);
  const setAdvancedFilters = usePlayerStore((s) => s.setAdvancedFilters);
  const defaultAdvancedFilters = usePlayerStore((s) => s.defaultAdvancedFilters);
  const setDefaultAdvancedFilters = usePlayerStore((s) => s.setDefaultAdvancedFilters);
  const [showPinModal, setShowPinModal] = useState(false);
  const [pendingChannel, setPendingChannel] = useState<Channel | null>(null);
  const parentRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<globalThis.HTMLInputElement>(null);
  const [showStalePrompt, setShowStalePrompt] = useState(false);
  const [stalePlaylistId, setStalePlaylistId] = useState<number | null>(null);
  const [showRefreshModal, setShowRefreshModal] = useState(false);

  // Global keyboard shortcuts (Space=play/stop, /=focus search, Escape=stop)
  useKeyboardShortcuts(searchInputRef);

  // Series progress tracking + auto-next before the end credits (desktop/MPV)
  const { autoNextIn, cancelAutoNext } = useSeriesProgress();

  // Responsive grid configuration
  const { columns, cardHeight, estimatedRowHeight } = useResponsiveGrid();

  // Load parental settings on mount
  useEffect(() => {
    loadParentalSettings();
  }, [loadParentalSettings]);

  // Load the default advanced filters (VOD/series) and apply them at startup
  useEffect(() => {
    getSetting(DEFAULT_FILTERS_SETTING)
      .then((raw) => {
        if (!raw) return;
        const defaults: AdvancedFilters = { ...EMPTY_ADVANCED_FILTERS, ...JSON.parse(raw) };
        setDefaultAdvancedFilters(defaults);
        setAdvancedFilters(defaults);
      })
      .catch((err) => logger.error('Failed to load default filters:', err));
  }, [setDefaultAdvancedFilters, setAdvancedFilters]);

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

  // On the Favorites tab, also fetch favorites from OTHER profiles for read-only display
  useEffect(() => {
    if (contentTypeFilter !== 'favorites') {
      setOtherProfileFavorites([]);
      return;
    }

    getFavorites()
      .then((allFavorites) => {
        const others = allFavorites.filter((c) => c.playlist_id !== currentPlaylist?.id);
        setOtherProfileFavorites(others);
      })
      .catch((err) => {
        logger.error('Failed to fetch cross-profile favorites:', err);
        setOtherProfileFavorites([]);
      });
  }, [contentTypeFilter, currentPlaylist?.id, setOtherProfileFavorites]);

  // Pre-compute parental blocking results (avoids per-card shouldBlockChannel calls)
  const blockedMap = useMemo(() => {
    if (!parentalEnabled || parentalUnlocked) return new Map<number, boolean>();

    const map = new Map<number, boolean>();
    for (const channel of filteredChannels) {
      if (channel.id) {
        map.set(
          channel.id,
          shouldBlockChannel(channel, {
            enabled: parentalEnabled,
            autoDetect: parentalAutoDetect,
            blockedIds: blockedChannelIds,
            blockedCategories: blockedCategories,
            unlocked: parentalUnlocked,
          })
        );
      }
    }
    return map;
  }, [
    filteredChannels,
    parentalEnabled,
    parentalUnlocked,
    parentalAutoDetect,
    blockedChannelIds,
    blockedCategories,
  ]);

  // Virtual scrolling setup - virtualize by rows (dynamic items per row)
  const rowCount = Math.ceil(filteredChannels.length / columns);

  const rowVirtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => parentRef.current,
    estimateSize: () => estimatedRowHeight,
    overscan: 3, // Render 3 extra rows above/below viewport
  });

  // Reset scroll position when switching tab/category, otherwise the previous
  // scrollTop can land past the new (shorter) list and it appears empty.
  useEffect(() => {
    rowVirtualizer.scrollToOffset(0);
  }, [contentTypeFilter, categoryFilter, rowVirtualizer]);

  const handlePlayChannel = useCallback(
    async (channel: Channel) => {
      // Check parental controls
      const isBlocked = shouldBlockChannel(channel, {
        enabled: parentalEnabled,
        autoDetect: parentalAutoDetect,
        blockedIds: blockedChannelIds,
        blockedCategories: blockedCategories,
        unlocked: parentalUnlocked,
      });

      if (isBlocked && parentalEnabled && !parentalUnlocked) {
        setPendingChannel(channel);
        setShowPinModal(true);
        return;
      }

      // VOD on Xtream playlist → show info view before playing
      if (
        channel.content_type === 'vod' &&
        currentPlaylist?.xtream_username &&
        currentPlaylist?.xtream_password
      ) {
        setSelectedVod(channel);
        return;
      }

      const result = await playChannelAction(channel);
      if (result?.type === 'series') {
        setSelectedSeries(result.channel);
      }
    },
    [
      parentalEnabled,
      parentalAutoDetect,
      blockedChannelIds,
      blockedCategories,
      parentalUnlocked,
      playChannelAction,
    ]
  );

  const handlePlayEpisode = useCallback(
    async (
      episodeId: string,
      extension: string,
      title: string,
      remainingEpisodes?: QueuedEpisode[],
      startSeconds?: number
    ) => {
      if (!currentPlaylist) return;
      try {
        await playEpisodeAction(
          episodeId,
          extension,
          title,
          currentPlaylist,
          remainingEpisodes,
          startSeconds
        );
      } catch (err) {
        logger.error('Failed to play episode:', err);
      }
    },
    [currentPlaylist, playEpisodeAction]
  );

  const handlePinSuccess = useCallback(() => {
    setShowPinModal(false);
    if (pendingChannel) {
      playChannelAction(pendingChannel)
        .then(() => {
          setPendingChannel(null);
        })
        .catch((err) => {
          logger.error('Failed to play channel after PIN:', err);
        });
    }
  }, [pendingChannel, playChannelAction]);

  const handleStop = useCallback(async () => {
    await stopPlaybackAction();
  }, [stopPlaybackAction]);

  // Now Playing bar (desktop only — iOS uses fullscreen IOSVideoPlayer)
  const nowPlayingBar =
    currentChannel && !isIOS() ? (
      <NowPlayingBar
        channel={currentChannel}
        currentProgram={currentProgram}
        nextProgram={nextProgram}
        onStop={handleStop}
        autoNextIn={autoNextIn}
        onCancelAutoNext={cancelAutoNext}
      />
    ) : null;

  // If a VOD is selected, show the VodView
  if (
    selectedVod &&
    currentPlaylist?.url &&
    currentPlaylist.xtream_username &&
    currentPlaylist.xtream_password
  ) {
    const urlParts = selectedVod.url?.split('/');
    const vodIdWithExt = urlParts?.[urlParts.length - 1];
    const vodId = vodIdWithExt ? parseInt(vodIdWithExt.replace(/\.\w+$/, ''), 10) : NaN;

    if (isNaN(vodId)) {
      logger.error('Failed to parse VOD ID from URL:', selectedVod.url);
      // Fallback: play directly
      playChannelAction(selectedVod);
      setSelectedVod(null);
      return null;
    }

    return (
      <VodView
        vodId={vodId}
        vodName={selectedVod.name}
        channelId={selectedVod.id}
        serverUrl={currentPlaylist.url}
        username={currentPlaylist.xtream_username}
        password={currentPlaylist.xtream_password}
        onBack={() => setSelectedVod(null)}
        onPlay={async () => {
          setSelectedVod(null);
          await playChannelAction(selectedVod);
        }}
      />
    );
  }

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
      <div className="flex h-screen flex-col">
        <div className="min-h-0 flex-1">
      <SeriesView
        seriesId={seriesId}
        seriesName={selectedSeries.name}
        channelId={selectedSeries.id}
        serverUrl={currentPlaylist.url}
        username={currentPlaylist.xtream_username}
        password={currentPlaylist.xtream_password}
        onBack={() => setSelectedSeries(null)}
        onPlayEpisode={handlePlayEpisode}
      />
        </div>
        {nowPlayingBar}
      </div>
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
      <ContentTypeTabs
        activeFilter={contentTypeFilter}
        onFilterChange={(f) => {
          setContentTypeFilter(f);
          setAdvancedFilters(defaultAdvancedFilters);
          setShowFilters(false);
        }}
      />

      {/* Filter button row — visible on vod/series tabs only */}
      {(contentTypeFilter === 'vod' || contentTypeFilter === 'series') && (() => {
        const activeCount = [
          advancedFilters.languages.length > 0,
          advancedFilters.minYear !== null || advancedFilters.maxYear !== null,
          advancedFilters.minRating !== null,
        ].filter(Boolean).length;
        return (
          <div className="flex items-center border-b border-gray-200 bg-white px-4 py-2 dark:border-gray-700 dark:bg-gray-800">
            <button
              onClick={() => setShowFilters((v) => !v)}
              className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium transition-colors ${
                showFilters || activeCount > 0
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
              }`}
            >
              <SlidersHorizontal className="h-4 w-4" />
              Filtres
              {activeCount > 0 && (
                <span className="flex h-4 w-4 items-center justify-center rounded-full bg-white text-xs font-bold text-blue-600">
                  {activeCount}
                </span>
              )}
            </button>
            {contentTypeFilter === 'series' && seriesChannels.length > 0 && (
              <button
                onClick={() => setShowRandomPlayModal(true)}
                className="ml-2 flex items-center gap-1.5 rounded-lg bg-gray-100 px-3 py-1.5 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600"
              >
                <Shuffle className="h-4 w-4" />
                Play aléatoire multi-séries
              </button>
            )}
          </div>
        );
      })()}

      {/* Advanced Filter Panel */}
      {showFilters && (contentTypeFilter === 'vod' || contentTypeFilter === 'series') && (
        <FilterPanel onClose={() => setShowFilters(false)} />
      )}

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
                        const isChannelBlocked = blockedMap.get(channel.id!) ?? false;

                        return (
                          <ChannelCard
                            key={channel.id}
                            channel={channel}
                            isPlaying={currentChannel?.id === channel.id && isPlaying}
                            onPlay={handlePlayChannel}
                            onToggleFavorite={toggleChannelFavorite}
                            currentProgram={channelEpgData.get(channel.id)}
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

        {/* Favorites from other profiles — read-only, visually distinct */}
        {contentTypeFilter === 'favorites' && otherProfileFavorites.length > 0 && (
          <div className="mx-auto border-t border-gray-200 p-4 dark:border-gray-700">
            <h2 className="mb-3 text-sm font-semibold text-gray-500 dark:text-gray-400">
              Favoris des autres profils (lecture seule)
            </h2>
            <div className={`grid ${getGridClasses(columns)} gap-4`}>
              {otherProfileFavorites.map((channel) => (
                <ChannelCard
                  key={`other-${channel.id}`}
                  channel={channel}
                  isPlaying={false}
                  onPlay={() => {}}
                  cardHeight={cardHeight}
                  readOnly
                  readOnlyLabel={
                    playlists.find((p) => p.id === channel.playlist_id)?.name ?? 'Autre profil'
                  }
                />
              ))}
            </div>
          </div>
        )}
      </div>

      {nowPlayingBar}

      {/* iOS fullscreen video player */}
      {isIOS() && isPlaying && currentChannel && (
        <IOSVideoPlayer
          channel={currentChannel}
          streamUrls={iosStreamUrls}
          currentProgram={currentProgram}
          onClose={handleStop}
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

      {/* Random play across multiple series */}
      {showRandomPlayModal &&
        currentPlaylist?.id != null &&
        currentPlaylist.url &&
        currentPlaylist.xtream_username &&
        currentPlaylist.xtream_password && (
          <RandomPlayModal
            isOpen={showRandomPlayModal}
            onClose={() => setShowRandomPlayModal(false)}
            seriesChannels={seriesChannels}
            playlistId={currentPlaylist.id}
            serverUrl={currentPlaylist.url}
            username={currentPlaylist.xtream_username}
            password={currentPlaylist.xtream_password}
            onPlay={handlePlayEpisode}
          />
        )}

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
