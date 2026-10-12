import { create } from 'zustand';
import type { AdvancedFilters, Channel, Playlist, QueuedEpisode, SeriesInfo } from '../types';
import { getParentalSettings, getBlockedChannels, toggleFavorite } from '../lib/tauri';

interface PlayerState {
  // Playlists
  playlists: Playlist[];
  currentPlaylist: Playlist | null;
  activeProfileId: number | null;
  setPlaylists: (playlists: Playlist[]) => void;
  setCurrentPlaylist: (playlist: Playlist | null) => void;
  setActiveProfileId: (id: number | null) => void;

  // Channels
  channels: Channel[];
  filteredChannels: Channel[];
  currentChannel: Channel | null;
  // Pre-filtered channels by type (for instant tab switching)
  liveChannels: Channel[];
  vodChannels: Channel[];
  seriesChannels: Channel[];
  favoriteChannels: Channel[];
  // Favorites belonging to OTHER profiles/playlists (read-only display, see Favorites tab)
  otherProfileFavorites: Channel[];
  setOtherProfileFavorites: (channels: Channel[]) => void;
  setChannels: (channels: Channel[]) => void;
  setFilteredChannels: (channels: Channel[]) => void;
  setCurrentChannel: (channel: Channel | null) => void;
  toggleChannelFavorite: (channelId: number) => Promise<void>;

  // Search
  searchQuery: string;
  setSearchQuery: (query: string) => void;

  // Content Type Filter
  contentTypeFilter: 'all' | 'live' | 'vod' | 'series' | 'favorites';
  setContentTypeFilter: (filter: 'all' | 'live' | 'vod' | 'series' | 'favorites') => void;

  // Category Filter (provider categories like "Sweden", "Norway", etc.)
  categoryFilter: string | null;
  categories: string[];
  setCategoryFilter: (category: string | null) => void;
  setCategories: (categories: string[]) => void;

  // Advanced Filters (vod/series only)
  advancedFilters: AdvancedFilters;
  setAdvancedFilters: (filters: AdvancedFilters) => void;
  // Filters applied automatically at startup and when switching tabs (saved in settings)
  defaultAdvancedFilters: AdvancedFilters;
  setDefaultAdvancedFilters: (filters: AdvancedFilters) => void;

  // Playback
  isPlaying: boolean;
  setIsPlaying: (playing: boolean) => void;
  // Episodes queued in MPV (same order as MPV's playlist), used to track series progress
  seriesSession: QueuedEpisode[] | null;
  setSeriesSession: (episodes: QueuedEpisode[] | null) => void;
  // Short feedback shown in the Now Playing bar (seek result, errors...)
  playbackNotice: { id: number; text: string; isError: boolean } | null;
  showPlaybackNotice: (text: string, isError?: boolean) => void;
  clearPlaybackNotice: () => void;

  // EPG (Electronic Program Guide)
  currentProgram: string | null;
  nextProgram: string | null;
  setCurrentProgram: (program: string | null) => void;
  setNextProgram: (program: string | null) => void;

  // EPG data for all channels (channelId -> current program)
  channelEpgData: Map<number, string>;
  setChannelEpg: (channelId: number, program: string | null) => void;
  clearAllEpg: () => void;
  epgRefreshTrigger: number;
  triggerEpgRefresh: () => void;

  // Series Navigation
  currentSeries: SeriesInfo | null;
  selectedSeason: string | null;
  setCurrentSeries: (series: SeriesInfo | null) => void;
  setSelectedSeason: (seasonNumber: string | null) => void;

  // UI State
  isSetupComplete: boolean;
  setIsSetupComplete: (complete: boolean) => void;
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;

  // Parental Controls
  parentalEnabled: boolean;
  parentalUnlocked: boolean;
  parentalUnlockExpiry: number | null;
  blockedChannelIds: Set<number>;
  blockedCategories: string[];
  parentalVisibility: 'hide' | 'lock' | 'blur';
  parentalAutoDetect: boolean;
  setParentalEnabled: (enabled: boolean) => void;
  setParentalUnlocked: (unlocked: boolean, duration?: number) => void;
  setBlockedChannelIds: (ids: Set<number>) => void;
  setBlockedCategories: (categories: string[]) => void;
  setParentalVisibility: (mode: 'hide' | 'lock' | 'blur') => void;
  setParentalAutoDetect: (enabled: boolean) => void;
  loadParentalSettings: () => Promise<void>;
  checkChannelBlocked: (channel: Channel) => boolean;
}

export const EMPTY_ADVANCED_FILTERS: AdvancedFilters = {
  languages: [],
  minYear: null,
  maxYear: null,
  minRating: null,
};

export const usePlayerStore = create<PlayerState>((set) => ({
  // Playlists
  playlists: [],
  currentPlaylist: null,
  activeProfileId: null,
  setPlaylists: (playlists) => set({ playlists }),
  setCurrentPlaylist: (playlist) => set({ currentPlaylist: playlist }),
  setActiveProfileId: (id) => set({ activeProfileId: id }),

  // Channels
  channels: [],
  filteredChannels: [],
  currentChannel: null,
  liveChannels: [],
  vodChannels: [],
  seriesChannels: [],
  favoriteChannels: [],
  otherProfileFavorites: [],
  setOtherProfileFavorites: (channels) => set({ otherProfileFavorites: channels }),
  setChannels: (channels) => {
    // Pre-filter channels by type for instant tab switching
    const liveChannels = channels.filter((c) => c.content_type === 'live');
    const vodChannels = channels.filter((c) => c.content_type === 'vod');
    const seriesChannels = channels.filter((c) => c.content_type === 'series');
    const favoriteChannels = channels.filter((c) => c.is_favorite);

    set({
      channels,
      filteredChannels: channels,
      liveChannels,
      vodChannels,
      seriesChannels,
      favoriteChannels,
    });
  },
  setFilteredChannels: (channels) => set({ filteredChannels: channels }),
  setCurrentChannel: (channel) => set({ currentChannel: channel }),

  toggleChannelFavorite: async (channelId) => {
    // Call backend IPC to persist the favorite toggle
    await toggleFavorite(channelId);

    // Update local state: rebuild only affected arrays
    set((state) => {
      const updatedChannels = state.channels.map((c) =>
        c.id === channelId ? { ...c, is_favorite: !c.is_favorite } : c
      );

      const targetChannel = state.channels.find((c) => c.id === channelId);
      const contentType = targetChannel?.content_type;

      const result: Partial<PlayerState> = {
        channels: updatedChannels,
        favoriteChannels: updatedChannels.filter((c) => c.is_favorite),
        filteredChannels: state.filteredChannels.map((c) =>
          c.id === channelId ? { ...c, is_favorite: !c.is_favorite } : c
        ),
      };

      // Only rebuild the specific content type array that contains this channel
      if (contentType === 'live') {
        result.liveChannels = updatedChannels.filter((c) => c.content_type === 'live');
      } else if (contentType === 'vod') {
        result.vodChannels = updatedChannels.filter((c) => c.content_type === 'vod');
      } else if (contentType === 'series') {
        result.seriesChannels = updatedChannels.filter((c) => c.content_type === 'series');
      }

      return result;
    });
  },

  // Search
  searchQuery: '',
  setSearchQuery: (query) => set({ searchQuery: query }),

  // Content Type Filter
  contentTypeFilter: 'live',
  setContentTypeFilter: (filter) => set({ contentTypeFilter: filter, categoryFilter: null }),

  // Category Filter
  categoryFilter: null,
  categories: [],
  setCategoryFilter: (category) => set({ categoryFilter: category }),
  setCategories: (categories) => set({ categories }),

  // Advanced Filters
  advancedFilters: { ...EMPTY_ADVANCED_FILTERS },
  setAdvancedFilters: (filters) => set({ advancedFilters: filters }),
  defaultAdvancedFilters: { ...EMPTY_ADVANCED_FILTERS },
  setDefaultAdvancedFilters: (filters) => set({ defaultAdvancedFilters: filters }),

  // Playback
  isPlaying: false,
  setIsPlaying: (playing) => set({ isPlaying: playing }),
  seriesSession: null,
  setSeriesSession: (episodes) => set({ seriesSession: episodes }),
  playbackNotice: null,
  showPlaybackNotice: (text, isError = false) =>
    set({ playbackNotice: { id: Date.now(), text, isError } }),
  clearPlaybackNotice: () => set({ playbackNotice: null }),

  // EPG (Electronic Program Guide)
  currentProgram: null,
  nextProgram: null,
  setCurrentProgram: (program) => set({ currentProgram: program }),
  setNextProgram: (program) => set({ nextProgram: program }),

  // EPG data for all channels
  channelEpgData: new Map(),
  setChannelEpg: (channelId, program) =>
    set((state) => {
      const newMap = new Map(state.channelEpgData);
      if (program) {
        newMap.set(channelId, program);
      } else {
        newMap.delete(channelId);
      }
      return { channelEpgData: newMap };
    }),
  clearAllEpg: () => set({ channelEpgData: new Map() }),
  epgRefreshTrigger: 0,
  triggerEpgRefresh: () => set((state) => ({ epgRefreshTrigger: state.epgRefreshTrigger + 1 })),

  // Series Navigation
  currentSeries: null,
  selectedSeason: null,
  setCurrentSeries: (series) => set({ currentSeries: series }),
  setSelectedSeason: (seasonNumber) => set({ selectedSeason: seasonNumber }),

  // UI State
  isSetupComplete: false,
  setIsSetupComplete: (complete) => set({ isSetupComplete: complete }),
  isLoading: false,
  setIsLoading: (loading) => set({ isLoading: loading }),

  // Parental Controls
  parentalEnabled: false,
  parentalUnlocked: false,
  parentalUnlockExpiry: null,
  blockedChannelIds: new Set(),
  blockedCategories: [],
  parentalVisibility: 'hide',
  parentalAutoDetect: false,

  setParentalEnabled: (enabled) => set({ parentalEnabled: enabled }),

  setParentalUnlocked: (unlocked, duration) => {
    const expiry = duration ? Date.now() + duration : null;
    set({ parentalUnlocked: unlocked, parentalUnlockExpiry: expiry });
  },

  setBlockedChannelIds: (ids) => set({ blockedChannelIds: ids }),
  setBlockedCategories: (categories) => set({ blockedCategories: categories }),
  setParentalVisibility: (mode) => set({ parentalVisibility: mode }),
  setParentalAutoDetect: (enabled) => set({ parentalAutoDetect: enabled }),

  loadParentalSettings: async () => {
    try {
      const settings = await getParentalSettings();
      const blockedIds = await getBlockedChannels();

      set({
        parentalEnabled: settings.enabled,
        blockedChannelIds: new Set(blockedIds),
        blockedCategories: settings.blocked_categories,
        parentalVisibility: settings.visibility,
        parentalAutoDetect: settings.auto_detect,
      });
    } catch (error) {
      console.error('Failed to load parental settings:', error);
    }
  },

  checkChannelBlocked: (channel) => {
    const state = usePlayerStore.getState();
    if (!state.parentalEnabled || state.parentalUnlocked) return false;
    if (channel.id && state.blockedChannelIds.has(channel.id)) return true;
    if (channel.group_name && state.blockedCategories.includes(channel.group_name)) return true;
    return false;
  },
}));
