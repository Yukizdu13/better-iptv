export interface Playlist {
  id?: number;
  name: string;
  url?: string;
  file_path?: string;
  last_updated?: string;
  auto_refresh: boolean;
  created_at?: string;
}

export interface Channel {
  id?: number;
  playlist_id: number;
  name: string;
  url: string;
  logo?: string;
  group_name?: string;
  epg_id?: string;
  tvg_name?: string;
  content_type: 'live' | 'vod' | 'series';
  is_favorite: boolean;
  sort_order: number;
  created_at?: string;
}

export interface AppSettings {
  theme: 'light' | 'dark';
  volume: number;
  quality: 'auto' | '1080p' | '720p' | '480p';
}
