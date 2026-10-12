import { invoke } from '@tauri-apps/api/core';
import type {
  Channel,
  Playlist,
  SeriesInfo,
  MergeResult,
  LastEpisode,
  RandomPlayPreset,
  PlaybackStatus,
} from '../types';

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

export async function mpvSeek(seconds: number): Promise<void> {
  return await invoke('mpv_seek', { seconds });
}

export async function mpvShowText(text: string, durationMs: number): Promise<void> {
  return await invoke('mpv_show_text', { text, durationMs });
}

export async function mpvTogglePause(): Promise<void> {
  return await invoke('mpv_toggle_pause');
}

export async function mpvPlaylistNext(): Promise<void> {
  return await invoke('mpv_playlist_next');
}

export async function mpvGetPlaybackStatus(): Promise<PlaybackStatus> {
  return await invoke('mpv_get_playback_status');
}

export async function isPlaying(): Promise<boolean> {
  return await invoke('is_playing');
}

// ========== Playlist Commands ==========

export async function importPlaylist(name: string, source: string): Promise<Playlist> {
  return await invoke('import_playlist', { name, source });
}

export async function importXtreamPlaylist(
  name: string,
  serverUrl: string,
  username: string,
  password: string
): Promise<Playlist> {
  return await invoke('import_xtream_playlist', {
    name,
    serverUrl,
    username,
    password,
  });
}

export async function getPlaylists(): Promise<Playlist[]> {
  return await invoke('get_playlists');
}

export async function deletePlaylist(id: number): Promise<void> {
  await invoke('delete_playlist', { id });
}

export async function refreshPlaylist(playlistId: number): Promise<MergeResult> {
  return await invoke('refresh_playlist', { playlistId });
}

export async function getStalePlaylistIds(): Promise<number[]> {
  return await invoke('get_stale_playlist_ids');
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

export async function getChannelGroups(
  playlistId: number,
  contentType?: string
): Promise<string[]> {
  return await invoke('get_channel_groups', { playlistId, contentType });
}

// ========== Series Commands ==========

export interface PlaylistEpisode {
  id: string;
  title: string;
  extension: string;
}

export async function getVodInfo(
  serverUrl: string,
  username: string,
  password: string,
  vodId: number
): Promise<import('../types').VodInfo> {
  return await invoke('get_vod_info', {
    serverUrl,
    username,
    password,
    vodId,
  });
}

export async function getSeriesInfo(
  serverUrl: string,
  username: string,
  password: string,
  seriesId: number
): Promise<SeriesInfo> {
  return await invoke('get_series_info', {
    serverUrl,
    username,
    password,
    seriesId,
  });
}

export async function playEpisodeWithSeason(
  serverUrl: string,
  username: string,
  password: string,
  episodes: PlaylistEpisode[],
  startSeconds?: number
): Promise<void> {
  return await invoke('play_episode_with_season', {
    serverUrl,
    username,
    password,
    // Only send what the backend needs (queue items may carry UI metadata)
    episodes: episodes.map(({ id, title, extension }) => ({ id, title, extension })),
    startSeconds: startSeconds ?? null,
  });
}

export async function getRandomPlayPresets(playlistId: number): Promise<RandomPlayPreset[]> {
  return await invoke('get_random_play_presets', { playlistId });
}

export async function createRandomPlayPreset(
  playlistId: number,
  name: string,
  seriesChannelIds: number[]
): Promise<number> {
  return await invoke('create_random_play_preset', {
    playlistId,
    name,
    seriesChannelIds,
  });
}

export async function updateRandomPlayPreset(
  id: number,
  name: string,
  seriesChannelIds: number[]
): Promise<void> {
  return await invoke('update_random_play_preset', { id, name, seriesChannelIds });
}

export async function deleteRandomPlayPreset(id: number): Promise<void> {
  return await invoke('delete_random_play_preset', { id });
}

export async function getLastEpisode(channelId: number): Promise<LastEpisode | null> {
  return await invoke('get_last_episode', { channelId });
}

export async function setLastEpisode(
  channelId: number,
  season: string,
  episodeId: string,
  episodeNum: number,
  title: string,
  positionSeconds?: number | null,
  durationSeconds?: number | null
): Promise<void> {
  return await invoke('set_last_episode', {
    channelId,
    season,
    episodeId,
    episodeNum,
    title,
    positionSeconds: positionSeconds ?? null,
    durationSeconds: durationSeconds ?? null,
  });
}

// ========== Settings Commands ==========

export async function getSetting(key: string): Promise<string | null> {
  return await invoke('get_setting', { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  await invoke('set_setting', { key, value });
}

// ========== Profile Management Commands ==========

export async function getActiveProfileId(): Promise<number | null> {
  return await invoke('get_active_profile_id');
}

export async function setActiveProfileId(profileId: number): Promise<void> {
  return await invoke('set_active_profile_id', { profileId });
}

export async function renamePlaylist(playlistId: number, newName: string): Promise<void> {
  return await invoke('rename_playlist', { playlistId, newName });
}

// ========== EPG Commands ==========

export async function fetchEpgData(epgUrl: string): Promise<number> {
  return await invoke('fetch_epg_data', { epgUrl });
}

export async function getChannelEpg(channelEpgId: string): Promise<[string | null, string | null]> {
  return await invoke('get_channel_epg', { channelEpgId });
}

export interface EpgStatus {
  has_url: boolean;
  last_fetched: string | null;
  program_count: number;
}

export interface EpgRefreshResult {
  success: boolean;
  programs_loaded: number;
  timestamp: string;
  error: string | null;
}

export async function getEpgStatus(): Promise<EpgStatus> {
  return await invoke('get_epg_status');
}

export async function forceRefreshEpg(): Promise<EpgRefreshResult> {
  return await invoke('force_refresh_epg');
}

// ========== Parental Controls Commands ==========

export async function setParentalPin(pin: string): Promise<void> {
  return await invoke('set_parental_pin', { pin });
}

export async function verifyParentalPin(pin: string): Promise<boolean> {
  return await invoke('verify_parental_pin', { pin });
}

export async function resetParentalPin(): Promise<void> {
  return await invoke('reset_parental_pin');
}

export async function getBlockedChannels(): Promise<number[]> {
  return await invoke('get_blocked_channels');
}

export async function setBlockedChannels(channelIds: number[]): Promise<void> {
  return await invoke('set_blocked_channels', { channelIds });
}

export interface ParentalSettings {
  enabled: boolean;
  has_pin: boolean;
  auto_detect: boolean;
  blocked_channels: number[];
  blocked_categories: string[];
  unlock_duration: string;
  visibility: 'hide' | 'lock' | 'blur';
}

export async function getParentalSettings(): Promise<ParentalSettings> {
  return await invoke('get_parental_settings');
}
