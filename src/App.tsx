import { useEffect } from 'react';
import Setup from './components/Setup';
import MainScreen from './components/MainScreen';
import { usePlayerStore } from './stores/player-store';
import { getPlaylists, getChannels } from './lib/tauri';

export default function App() {
  const { isSetupComplete, setIsSetupComplete, setPlaylists, setChannels, setCurrentPlaylist } = usePlayerStore();

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
        console.error('Failed to check setup:', err);
      }
    }

    checkSetup();
  }, [setIsSetupComplete, setPlaylists, setChannels, setCurrentPlaylist]);

  return isSetupComplete ? <MainScreen /> : <Setup />;
}
