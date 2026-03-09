import { describe, it, expect, beforeEach } from 'vitest';
import { usePlayerStore } from '../../stores/player-store';
import type { Channel } from '../../types';

const makeChannel = (overrides: Partial<Channel>): Channel => ({
  id: 1,
  name: 'Test',
  url: 'http://test',
  playlist_id: 1,
  content_type: 'live',
  is_favorite: false,
  sort_order: 0,
  ...overrides,
});

describe('channel filtering logic', () => {
  beforeEach(() => {
    usePlayerStore.setState({
      channels: [],
      filteredChannels: [],
      liveChannels: [],
      vodChannels: [],
      seriesChannels: [],
      favoriteChannels: [],
      searchQuery: '',
      contentTypeFilter: 'all',
      categoryFilter: null,
    });
  });

  it('should pre-filter channels by content type on setChannels', () => {
    const channels = [
      makeChannel({ id: 1, name: 'Live 1', content_type: 'live' }),
      makeChannel({ id: 2, name: 'Movie 1', content_type: 'vod' }),
      makeChannel({ id: 3, name: 'Series 1', content_type: 'series' }),
    ];

    usePlayerStore.getState().setChannels(channels);
    const state = usePlayerStore.getState();

    expect(state.liveChannels).toHaveLength(1);
    expect(state.vodChannels).toHaveLength(1);
    expect(state.seriesChannels).toHaveLength(1);
  });

  it('should track favorite channels separately', () => {
    const channels = [
      makeChannel({ id: 1, name: 'Fav', is_favorite: true }),
      makeChannel({ id: 2, name: 'Not Fav', is_favorite: false }),
    ];

    usePlayerStore.getState().setChannels(channels);
    expect(usePlayerStore.getState().favoriteChannels).toHaveLength(1);
    expect(usePlayerStore.getState().favoriteChannels[0].name).toBe('Fav');
  });
});
