import { useEffect, useMemo } from 'react';
import { usePlayerStore } from '../stores/player-store';
import { shouldBlockChannel } from '../lib/parentalControls';
import type { Channel } from '../types';

/** Extrait le code langue du préfixe `|FR|` / `|EN|` etc. dans le nom. */
export function extractLanguage(name: string): string | null {
  const m = name.match(/^\|([A-Z]{2,3})\|/);
  return m ? m[1] : null;
}

/** Extrait l'année entre parenthèses à la fin du nom : `Titre (2013)` → 2013. */
export function extractYear(name: string): number | null {
  const m = name.match(/\((\d{4})\)\s*(?:\|\w+\|)?$/);
  if (!m) return null;
  const y = parseInt(m[1]);
  return y >= 1900 && y <= 2100 ? y : null;
}

/**
 * Consolidated channel filtering hook.
 *
 * Applies filters in order:
 * 1. Content type (instant via pre-filtered arrays)
 * 2. Category filter
 * 3. Parental controls (hide mode)
 * 4. Search query
 *
 * Syncs result to store via setFilteredChannels.
 */
export function useChannelFilter(debouncedSearchQuery: string): Channel[] {
  const channels = usePlayerStore((s) => s.channels);
  const filteredChannels = usePlayerStore((s) => s.filteredChannels);
  const liveChannels = usePlayerStore((s) => s.liveChannels);
  const vodChannels = usePlayerStore((s) => s.vodChannels);
  const seriesChannels = usePlayerStore((s) => s.seriesChannels);
  const favoriteChannels = usePlayerStore((s) => s.favoriteChannels);
  const contentTypeFilter = usePlayerStore((s) => s.contentTypeFilter);
  const categoryFilter = usePlayerStore((s) => s.categoryFilter);
  const setFilteredChannels = usePlayerStore((s) => s.setFilteredChannels);
  const advancedFilters = usePlayerStore((s) => s.advancedFilters);
  const parentalEnabled = usePlayerStore((s) => s.parentalEnabled);
  const parentalUnlocked = usePlayerStore((s) => s.parentalUnlocked);
  const blockedChannelIds = usePlayerStore((s) => s.blockedChannelIds);
  const blockedCategories = usePlayerStore((s) => s.blockedCategories);
  const parentalAutoDetect = usePlayerStore((s) => s.parentalAutoDetect);
  const parentalVisibility = usePlayerStore((s) => s.parentalVisibility);

  // Step 1: Select base list by content type (O(1) - pre-computed)
  const baseList = useMemo(() => {
    switch (contentTypeFilter) {
      case 'live':
        return liveChannels;
      case 'vod':
        return vodChannels;
      case 'series':
        return seriesChannels;
      case 'favorites':
        return favoriteChannels;
      default:
        return channels;
    }
  }, [contentTypeFilter, liveChannels, vodChannels, seriesChannels, favoriteChannels, channels]);

  // Step 2-4: Apply category, parental, and search filters
  useEffect(() => {
    let result = baseList;

    // Category filter
    if (categoryFilter) {
      result = result.filter((c) => c.group_name === categoryFilter);
    }

    // Parental controls (hide mode only)
    if (parentalEnabled && !parentalUnlocked && parentalVisibility === 'hide') {
      result = result.filter(
        (c) =>
          !shouldBlockChannel(c, {
            enabled: parentalEnabled,
            autoDetect: parentalAutoDetect,
            blockedIds: blockedChannelIds,
            blockedCategories: blockedCategories,
            unlocked: parentalUnlocked,
          })
      );
    }

    // Search filter
    if (debouncedSearchQuery.trim() !== '') {
      const query = debouncedSearchQuery.toLowerCase();
      result = result.filter(
        (c) => c.name.toLowerCase().includes(query) || c.group_name?.toLowerCase().includes(query)
      );
    }

    // Advanced filters (vod/series only — ignored for live/favorites)
    const { languages, minYear, maxYear, minRating } = advancedFilters;
    const isVodOrSeries = contentTypeFilter === 'vod' || contentTypeFilter === 'series';
    if (isVodOrSeries) {
      if (languages.length > 0) {
        result = result.filter((c) => {
          const lang = extractLanguage(c.name);
          return lang !== null && languages.includes(lang);
        });
      }
      if (minYear !== null) {
        result = result.filter((c) => {
          const y = extractYear(c.name);
          return y !== null && y >= minYear;
        });
      }
      if (maxYear !== null) {
        result = result.filter((c) => {
          const y = extractYear(c.name);
          return y !== null && y <= maxYear;
        });
      }
      if (minRating !== null) {
        result = result.filter((c) => c.rating != null && c.rating >= minRating);
      }
    }

    setFilteredChannels(result);
  }, [
    baseList,
    categoryFilter,
    debouncedSearchQuery,
    parentalEnabled,
    parentalUnlocked,
    parentalAutoDetect,
    blockedChannelIds,
    blockedCategories,
    parentalVisibility,
    setFilteredChannels,
    advancedFilters,
    contentTypeFilter,
  ]);

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
  const contentTypeFilter = usePlayerStore((s) => s.contentTypeFilter);
  const setContentTypeFilter = usePlayerStore((s) => s.setContentTypeFilter);

  return {
    activeFilter: contentTypeFilter as ContentTypeFilter,
    setFilter: setContentTypeFilter,
  };
}

/**
 * Hook to manage search query state
 */
export function useSearchQuery() {
  const searchQuery = usePlayerStore((s) => s.searchQuery);
  const setSearchQuery = usePlayerStore((s) => s.setSearchQuery);

  return {
    query: searchQuery,
    setQuery: setSearchQuery,
  };
}
