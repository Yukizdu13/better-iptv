import { useEffect, useState } from 'react';
import Setup from './components/Setup';
import MainScreen from './components/MainScreen';
import { usePlayerStore } from './stores/player-store';
import { getPlaylists, getChannels } from './lib/tauri';
import { logger } from './lib/logger';

export default function App() {
  const { isSetupComplete, setIsSetupComplete, setPlaylists, setChannels, setCurrentPlaylist } = usePlayerStore();
  const [isCheckingSetup, setIsCheckingSetup] = useState(true);

  useEffect(() => {
    // Check if user has any playlists on app start
    async function checkSetup() {
      try {
        const playlists = await getPlaylists();

        if (playlists.length > 0) {
          // User has playlists, load the first one
          setPlaylists(playlists);
          setCurrentPlaylist(playlists[0]);

          const channels = await getChannels(playlists[0].id);
          setChannels(channels);
          setIsSetupComplete(true);
        }
      } catch (err) {
        logger.error('Failed to check setup:', err);
      } finally {
        setIsCheckingSetup(false);
      }
    }

    checkSetup();
  }, [setIsSetupComplete, setPlaylists, setChannels, setCurrentPlaylist]);

  if (isCheckingSetup) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-4 border-blue-500 border-t-transparent mx-auto mb-4"></div>
          <p className="text-gray-700 dark:text-gray-300 font-medium text-lg">
            Loading playlist...
          </p>
        </div>
      </div>
    );
  }

  return isSetupComplete ? <MainScreen /> : <Setup />;
}
