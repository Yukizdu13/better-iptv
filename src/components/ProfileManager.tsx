import { useState } from 'react';
import { usePlayerStore } from '../stores/player-store';
import {
  setActiveProfileId,
  deletePlaylist,
  renamePlaylist,
  getChannels
} from '../lib/tauri';
import { logger } from '../lib/logger';
import Setup from './Setup';
import type { Playlist } from '../types';

interface ProfileManagerProps {
  onClose: () => void; // For closing Settings modal if needed
}

export default function ProfileManager({ onClose }: ProfileManagerProps) {
  const {
    playlists,
    activeProfileId,
    setActiveProfileId: setStoreActiveId,
    setCurrentPlaylist,
    setChannels,
    setIsSetupComplete,
  } = usePlayerStore();

  const [showSetupModal, setShowSetupModal] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editName, setEditName] = useState('');
  const [showDeleteWarning, setShowDeleteWarning] = useState<number | null>(null);

  // Switch to a different profile
  const handleActivateProfile = async (playlist: Playlist) => {
    try {
      logger.info(`Switching to profile: ${playlist.name}`);

      // Set active in backend
      await setActiveProfileId(playlist.id!);

      // Load channels for this playlist
      const channels = await getChannels(playlist.id!);

      // Update frontend state
      setStoreActiveId(playlist.id!);
      setCurrentPlaylist(playlist);
      setChannels(channels);

      logger.info(`Profile switched successfully: ${channels.length} channels loaded`);
    } catch (err) {
      logger.error('Failed to switch profile:', err);
      alert(`Failed to switch profile: ${err}`);
    }
  };

  // Start rename process
  const handleStartRename = (playlist: Playlist) => {
    setEditingId(playlist.id!);
    setEditName(playlist.name);
  };

  // Save renamed profile
  const handleSaveRename = async (id: number) => {
    if (!editName.trim()) {
      alert('Profile name cannot be empty');
      return;
    }

    try {
      await renamePlaylist(id, editName.trim());

      // Update playlists in store
      const updatedPlaylists = playlists.map(p =>
        p.id === id ? { ...p, name: editName.trim() } : p
      );
      usePlayerStore.setState({ playlists: updatedPlaylists });

      setEditingId(null);
      logger.info(`Profile ID ${id} renamed to: ${editName.trim()}`);
    } catch (err) {
      logger.error('Failed to rename profile:', err);
      alert(`Failed to rename profile: ${err}`);
    }
  };

  // Cancel rename
  const handleCancelRename = () => {
    setEditingId(null);
    setEditName('');
  };

  // Delete profile with special logic
  const handleDeleteProfile = async (id: number) => {
    const isActive = id === activeProfileId;
    const isLastProfile = playlists.length === 1;

    if (isLastProfile) {
      // Show warning modal for last profile
      setShowDeleteWarning(id);
      return;
    }

    if (isActive) {
      // Deleting active profile, need to switch first
      const remainingPlaylists = playlists.filter(p => p.id !== id);
      const nextProfile = remainingPlaylists[0];

      // Switch to next profile first
      await handleActivateProfile(nextProfile);
    }

    // Now delete the profile
    try {
      await deletePlaylist(id);

      // Update store
      const updatedPlaylists = playlists.filter(p => p.id !== id);
      usePlayerStore.setState({ playlists: updatedPlaylists });

      logger.info(`Profile ID ${id} deleted`);
    } catch (err) {
      logger.error('Failed to delete profile:', err);
      alert(`Failed to delete profile: ${err}`);
    }
  };

  // Confirm delete last profile
  const handleConfirmDeleteLastProfile = async () => {
    const id = showDeleteWarning!;

    try {
      await deletePlaylist(id);

      // Reset to setup screen
      setIsSetupComplete(false);
      setShowDeleteWarning(null);
      onClose(); // Close Settings modal

      logger.info('Last profile deleted, returning to setup');
    } catch (err) {
      logger.error('Failed to delete last profile:', err);
      alert(`Failed to delete profile: ${err}`);
    }
  };

  // Handle new profile creation
  const handleProfileCreated = async (newPlaylist: Playlist) => {
    setShowSetupModal(false);

    // Add to playlists
    const updatedPlaylists = [...playlists, newPlaylist];
    usePlayerStore.setState({ playlists: updatedPlaylists });

    // Auto-activate new profile
    await handleActivateProfile(newPlaylist);
  };

  return (
    <>
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Profiles
          </h3>
          <button
            onClick={() => setShowSetupModal(true)}
            className="flex items-center gap-2 bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition-colors"
          >
            <span className="text-xl">+</span>
            Create New Profile
          </button>
        </div>

        <div className="space-y-3">
          {playlists.map((playlist) => {
            const isActive = playlist.id === activeProfileId;
            const isEditing = editingId === playlist.id;
            const type = playlist.xtream_username ? 'Xtream Codes' : 'M3U URL';
            const icon = type === 'Xtream Codes' ? '📡' : '📺';

            return (
              <div
                key={playlist.id}
                className={`p-4 rounded-lg border-2 transition-all ${
                  isActive
                    ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-500'
                    : 'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3 flex-1">
                    <div className="text-2xl">{icon}</div>
                    <div className="flex-1">
                      {isEditing ? (
                        <input
                          type="text"
                          value={editName}
                          onChange={(e) => setEditName(e.target.value)}
                          className="w-full px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md dark:bg-gray-700 dark:text-white"
                          autoFocus
                          onKeyDown={(e) => {
                            if (e.key === 'Enter') handleSaveRename(playlist.id!);
                            if (e.key === 'Escape') handleCancelRename();
                          }}
                        />
                      ) : (
                        <h3 className="font-semibold text-gray-900 dark:text-white">
                          {playlist.name}
                        </h3>
                      )}
                      <p className="text-sm text-gray-600 dark:text-gray-400">
                        Type: {type}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    {isActive ? (
                      <span className="bg-blue-600 text-white px-3 py-1 rounded-full text-sm font-medium">
                        Active
                      </span>
                    ) : (
                      <button
                        onClick={() => handleActivateProfile(playlist)}
                        className="bg-green-600 hover:bg-green-700 text-white px-3 py-1 rounded-md text-sm font-medium transition-colors"
                      >
                        Activate
                      </button>
                    )}

                    {isEditing ? (
                      <>
                        <button
                          onClick={() => handleSaveRename(playlist.id!)}
                          className="bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded-md text-sm"
                        >
                          Save
                        </button>
                        <button
                          onClick={handleCancelRename}
                          className="bg-gray-500 hover:bg-gray-600 text-white px-3 py-1 rounded-md text-sm"
                        >
                          Cancel
                        </button>
                      </>
                    ) : (
                      <>
                        <button
                          onClick={() => handleStartRename(playlist)}
                          className="bg-gray-600 hover:bg-gray-700 text-white px-3 py-1 rounded-md text-sm"
                        >
                          Rename
                        </button>
                        <button
                          onClick={() => handleDeleteProfile(playlist.id!)}
                          className="bg-red-600 hover:bg-red-700 text-white px-3 py-1 rounded-md text-sm"
                        >
                          Delete
                        </button>
                      </>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Setup Modal for Creating New Profile */}
      {showSetupModal && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-[60] flex items-center justify-center p-4">
          <Setup
            onComplete={handleProfileCreated}
            onCancel={() => setShowSetupModal(false)}
          />
        </div>
      )}

      {/* Delete Last Profile Warning Modal */}
      {showDeleteWarning !== null && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-[70] flex items-center justify-center p-4">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md">
            <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
              Delete Last Profile?
            </h3>
            <p className="text-gray-700 dark:text-gray-300 mb-6">
              This is your only profile. If you delete it, the onboarding process will start again
              and you'll need to add a new playlist.
            </p>
            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setShowDeleteWarning(null)}
                className="px-4 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-lg"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmDeleteLastProfile}
                className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg"
              >
                Delete and Restart
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
