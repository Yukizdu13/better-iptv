import { useState, useEffect } from 'react';
import { importPlaylist, importXtreamPlaylist, getChannels } from '../lib/tauri';
import { usePlayerStore } from '../stores/player-store';
import { listen } from '@tauri-apps/api/event';
import { logger } from '../lib/logger';
import type { Playlist } from '../types';

type ImportType = 'm3u' | 'xtream';

interface SetupProps {
  onComplete?: (playlist: Playlist) => void; // Callback when profile created (modal mode)
  onCancel?: () => void; // Callback to cancel modal
}

export default function Setup({ onComplete, onCancel }: SetupProps = {}) {
  const [importType, setImportType] = useState<ImportType>('m3u');
  const [playlistName, setPlaylistName] = useState('');

  // M3U fields
  const [playlistUrl, setPlaylistUrl] = useState('');

  // Xtream fields
  const [serverUrl, setServerUrl] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');

  const [error, setError] = useState('');
  const [importProgress, setImportProgress] = useState<{
    live_count: number;
    vod_count: number;
    series_count: number;
  } | null>(null);

  const { setIsSetupComplete, setChannels, setIsLoading, setCurrentPlaylist, isLoading } =
    usePlayerStore();

  // Listen for import progress events
  useEffect(() => {
    const unlisten = listen<{ live_count: number; vod_count: number; series_count: number }>(
      'import-progress',
      (event) => {
        setImportProgress(event.payload);
      }
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!playlistName.trim()) {
      setError('Please enter a playlist name');
      return;
    }

    if (importType === 'm3u' && !playlistUrl.trim()) {
      setError('Please enter a playlist URL');
      return;
    }

    if (importType === 'xtream' && (!serverUrl.trim() || !username.trim() || !password.trim())) {
      setError('Please fill in all Xtream credentials');
      return;
    }

    setIsLoading(true);
    setImportProgress(null);

    try {
      let playlist;

      if (importType === 'm3u') {
        logger.info('Importing M3U playlist...');
        playlist = await importPlaylist(playlistName, playlistUrl);
      } else {
        logger.info('Importing Xtream playlist...', { serverUrl, username });
        playlist = await importXtreamPlaylist(playlistName, serverUrl, username, password);
        logger.debug('Xtream import result:', playlist);
      }

      logger.debug('Fetching channels for playlist:', playlist.id);
      const channels = await getChannels(playlist.id);
      logger.info('Fetched channels:', channels.length);

      // If modal mode with onComplete callback, call it
      if (onComplete) {
        onComplete(playlist);
        return; // Don't set setup complete, parent handles state
      }

      // Normal mode (initial setup)
      setCurrentPlaylist(playlist);
      setChannels(channels);
      setIsSetupComplete(true);
    } catch (err) {
      logger.error('Import error:', err);
      setError(err instanceof Error ? err.message : 'Failed to import playlist');
    } finally {
      setIsLoading(false);
      setImportProgress(null);
    }
  };

  return (
    <div
      className={
        onCancel
          ? ''
          : 'flex min-h-screen items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 p-4 dark:from-gray-900 dark:to-gray-800'
      }
    >
      <div
        className={`rounded-lg bg-white p-8 shadow-xl dark:bg-gray-800 ${onCancel ? 'w-full max-w-md' : 'w-full max-w-md'} relative`}
      >
        {isLoading && (
          <div className="absolute inset-0 z-10 flex items-center justify-center rounded-lg bg-white/80 backdrop-blur-sm dark:bg-gray-800/80">
            <div className="text-center">
              <div className="mx-auto mb-4 h-16 w-16 animate-spin rounded-full border-4 border-blue-500 border-t-transparent"></div>
              <p className="font-medium text-gray-700 dark:text-gray-300">Importing playlist...</p>
              {importProgress && (
                <div className="mt-3 space-y-1 text-sm text-gray-600 dark:text-gray-400">
                  {importProgress.live_count > 0 && (
                    <p>Fetched {importProgress.live_count.toLocaleString()} live streams</p>
                  )}
                  {importProgress.vod_count > 0 && (
                    <p>Fetched {importProgress.vod_count.toLocaleString()} VOD streams</p>
                  )}
                  {importProgress.series_count > 0 && (
                    <p>Fetched {importProgress.series_count.toLocaleString()} series</p>
                  )}
                </div>
              )}
              {!importProgress && (
                <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">
                  This may take a few moments
                </p>
              )}
            </div>
          </div>
        )}

        {/* Cancel button if in modal mode */}
        {onCancel && (
          <button
            onClick={onCancel}
            className="absolute right-4 top-4 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            ✕
          </button>
        )}

        <div className="mb-8 text-center">
          <h1 className="mb-2 text-3xl font-bold text-gray-900 dark:text-white">
            {onCancel ? 'Add New Profile' : 'Better IPTV'}
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            {onCancel ? 'Add a new IPTV playlist' : 'Add your IPTV playlist to get started'}
          </p>
        </div>

        {/* Tab switcher */}
        <div className="mb-6 flex border-b border-gray-200 dark:border-gray-700">
          <button
            type="button"
            onClick={() => setImportType('m3u')}
            className={`flex-1 px-4 py-2 text-sm font-medium transition-colors ${
              importType === 'm3u'
                ? 'border-b-2 border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400'
                : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
            }`}
          >
            M3U URL
          </button>
          <button
            type="button"
            onClick={() => setImportType('xtream')}
            className={`flex-1 px-4 py-2 text-sm font-medium transition-colors ${
              importType === 'xtream'
                ? 'border-b-2 border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400'
                : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
            }`}
          >
            Xtream Codes
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label
              htmlFor="name"
              className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              Playlist Name
            </label>
            <input
              id="name"
              type="text"
              value={playlistName}
              onChange={(e) => setPlaylistName(e.target.value)}
              placeholder="My IPTV Playlist"
              className="w-full rounded-md border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            />
          </div>

          {importType === 'm3u' ? (
            <div>
              <label
                htmlFor="url"
                className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
              >
                M3U Playlist URL
              </label>
              <input
                id="url"
                type="text"
                value={playlistUrl}
                onChange={(e) => setPlaylistUrl(e.target.value)}
                placeholder="http://example.com/playlist.m3u"
                className="w-full rounded-md border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
              />
            </div>
          ) : (
            <>
              <div>
                <label
                  htmlFor="server"
                  className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  Server URL
                </label>
                <input
                  id="server"
                  type="text"
                  value={serverUrl}
                  onChange={(e) => setServerUrl(e.target.value)}
                  placeholder="http://example.com:8080"
                  className="w-full rounded-md border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                />
              </div>
              <div>
                <label
                  htmlFor="username"
                  className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  Username
                </label>
                <input
                  id="username"
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder="username"
                  className="w-full rounded-md border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                />
              </div>
              <div>
                <label
                  htmlFor="password"
                  className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  Password
                </label>
                <input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="password"
                  className="w-full rounded-md border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                />
              </div>
            </>
          )}

          {error && (
            <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-900/20 dark:text-red-400">
              {error}
            </div>
          )}

          <button
            type="submit"
            className="w-full rounded-md bg-blue-600 px-4 py-3 font-medium text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Add Playlist
          </button>
        </form>

        <div className="mt-6 text-center text-xs text-gray-500 dark:text-gray-400">
          Your playlist will be saved locally
        </div>
      </div>
    </div>
  );
}
