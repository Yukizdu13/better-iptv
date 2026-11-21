import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ChannelCard } from '../../components/ChannelCard';
import type { Channel } from '../../types';

function makeChannel(overrides: Partial<Channel> = {}): Channel {
  return {
    id: 1,
    playlist_id: 1,
    name: 'Film Test',
    url: 'http://example.com/movie/u/p/1.mkv',
    logo: 'https://img.example.com/poster.jpg',
    content_type: 'vod',
    is_favorite: false,
    sort_order: 0,
    category_order: 0,
    ...overrides,
  } as Channel;
}

describe('ChannelCard — image du logo', () => {
  it("affiche l'image quand le logo est fourni", () => {
    render(
      <ChannelCard channel={makeChannel()} isPlaying={false} onPlay={() => {}} cardHeight={240} />
    );
    expect(screen.getByRole('img', { name: 'Film Test' })).toBeTruthy();
  });

  it("replie sur l'initiale quand l'image ne charge pas (lien mort)", () => {
    render(
      <ChannelCard channel={makeChannel()} isPlaying={false} onPlay={() => {}} cardHeight={240} />
    );
    fireEvent.error(screen.getByRole('img', { name: 'Film Test' }));
    expect(screen.queryByRole('img', { name: 'Film Test' })).toBeNull();
    expect(screen.getByText('F')).toBeTruthy();
  });

  it('retente un nouveau logo quand la carte est réutilisée pour une autre chaîne', () => {
    const { rerender } = render(
      <ChannelCard channel={makeChannel()} isPlaying={false} onPlay={() => {}} cardHeight={240} />
    );
    fireEvent.error(screen.getByRole('img', { name: 'Film Test' }));
    rerender(
      <ChannelCard
        channel={makeChannel({ id: 2, name: 'Autre', logo: 'https://img.example.com/other.jpg' })}
        isPlaying={false}
        onPlay={() => {}}
        cardHeight={240}
      />
    );
    expect(screen.getByRole('img', { name: 'Autre' })).toBeTruthy();
  });
});
