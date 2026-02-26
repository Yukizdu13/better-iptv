import { useEffect, useMemo } from 'react';
import { usePlayerStore } from '../stores/player-store';
import type { Channel } from '../types';

/**
 * Filter channels based on search query and content type
 *
 * This hook consolidates the channel filtering logic that was previously
 * scattered across MainScreen. It uses pre-filtered lists for instant
 * tab switching and applies search filter on top.
 *
 * @returns The filtered channels list (automatically synced to store)
 */
export function useChannelFilter(): Channel[] {
  const {
    channels,
    filteredChannels,
    liveChannels,
    vodChannels,
    seriesChannels,
    searchQuery,
    contentTypeFilter,
    setFilteredChannels,
  } = usePlayerStore();

  // Get base list based on content type filter (instant tab switching)
  const baseList = useMemo(() => {
    switch (contentTypeFilter) {
      case 'live':
        return liveChannels;
      case 'vod':
        return vodChannels;
      case 'series':
        return seriesChannels;
      default:
        return channels;
    }
  }, [contentTypeFilter, liveChannels, vodChannels, seriesChannels, channels]);

  // Apply search filter on top of content-type filtered list
  useEffect(() => {
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
  }, [searchQuery, baseList, setFilteredChannels]);

  return filteredChannels;
}

/**
 * Content type filter options
 */
export type ContentTypeFilter = 'all' | 'live' | 'vod' | 'series' | 'favorites';

/**
 * Hook to manage content type filter state
 */
export function useContentTypeFilter() {
  const { contentTypeFilter, setContentTypeFilter } = usePlayerStore();

  return {
    activeFilter: contentTypeFilter as ContentTypeFilter,
    setFilter: setContentTypeFilter,
  };
}

/**
 * Hook to manage search query state
 */
export function useSearchQuery() {
  const { searchQuery, setSearchQuery } = usePlayerStore();

  return {
    query: searchQuery,
    setQuery: setSearchQuery,
  };
}
