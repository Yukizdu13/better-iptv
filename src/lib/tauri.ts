import { invoke } from '@tauri-apps/api/core';
import type { Channel, Playlist } from '../types';

// ========== MPV Commands ==========

export async function checkMpvInstalled(): Promise<boolean> {
  return await invoke('check_mpv_installed');
}

export async function playChannel(channel: Channel): Promise<void> {
  await invoke('play_channel', { channel });
}

export async function stopPlayback(): Promise<void> {
  await invoke('stop_playback');
}

export async function isPlaying(): Promise<boolean> {
  return await invoke('is_playing');
}

// ========== Playlist Commands ==========

export async function importPlaylist(name: string, source: string): Promise<Playlist> {
  return await invoke('import_playlist', { name, source });
}

export async function getPlaylists(): Promise<Playlist[]> {
  return await invoke('get_playlists');
}

export async function deletePlaylist(id: number): Promise<void> {
  await invoke('delete_playlist', { id });
}

// ========== Channel Commands ==========

export async function getChannels(playlistId?: number): Promise<Channel[]> {
  return await invoke('get_channels', { playlistId });
}

export async function searchChannels(query: string): Promise<Channel[]> {
  return await invoke('search_channels', { query });
}

export async function toggleFavorite(channelId: number): Promise<void> {
  await invoke('toggle_favorite', { channelId });
}

export async function getFavorites(): Promise<Channel[]> {
  return await invoke('get_favorites');
}

// ========== Settings Commands ==========

export async function getSetting(key: string): Promise<string | null> {
  return await invoke('get_setting', { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  await invoke('set_setting', { key, value });
}
