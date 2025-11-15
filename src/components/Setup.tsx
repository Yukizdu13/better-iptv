import { useState } from 'react';
import { importPlaylist, getChannels } from '../lib/tauri';
import { usePlayerStore } from '../stores/player-store';

export default function Setup() {
  const [playlistName, setPlaylistName] = useState('');
  const [playlistUrl, setPlaylistUrl] = useState('');
  const [error, setError] = useState('');

  const { setIsSetupComplete, setChannels, setIsLoading, setCurrentPlaylist } = usePlayerStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!playlistName.trim() || !playlistUrl.trim()) {
      setError('Please fill in both fields');
      return;
    }

    setIsLoading(true);

    try {
      // Import playlist and parse channels
      const playlist = await importPlaylist(playlistName, playlistUrl);

      // Load channels from database
      const channels = await getChannels(playlist.id);

      // Update store
      setCurrentPlaylist(playlist);
      setChannels(channels);
      setIsSetupComplete(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to import playlist');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-8 max-w-md w-full">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Better IPTV
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Add your IPTV playlist to get started
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="name" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Playlist Name
            </label>
            <input
              id="name"
              type="text"
              value={playlistName}
              onChange={(e) => setPlaylistName(e.target.value)}
              placeholder="My IPTV Playlist"
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
            />
          </div>

          <div>
            <label htmlFor="url" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              M3U Playlist URL
            </label>
            <input
              id="url"
              type="text"
              value={playlistUrl}
              onChange={(e) => setPlaylistUrl(e.target.value)}
              placeholder="http://example.com/playlist.m3u"
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
            />
          </div>

          {error && (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded-md text-sm">
              {error}
            </div>
          )}

          <button
            type="submit"
            className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-4 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
