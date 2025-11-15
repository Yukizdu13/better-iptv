import { useEffect } from 'react';
import { usePlayerStore } from '../stores/player-store';
import { playChannel, stopPlayback } from '../lib/tauri';
import { Search, Play, Square, Star } from 'lucide-react';

export default function MainScreen() {
  const {
    channels,
    filteredChannels,
    searchQuery,
    currentChannel,
    isPlaying,
    setSearchQuery,
    setFilteredChannels,
    setCurrentChannel,
    setIsPlaying,
  } = usePlayerStore();

  // Filter channels when search query changes
  useEffect(() => {
    if (searchQuery.trim() === '') {
      setFilteredChannels(channels);
    } else {
      const query = searchQuery.toLowerCase();
      const filtered = channels.filter(
        (channel) =>
          channel.name.toLowerCase().includes(query) ||
          channel.group_name?.toLowerCase().includes(query)
      );
      setFilteredChannels(filtered);
    }
  }, [searchQuery, channels, setFilteredChannels]);

  const handlePlayChannel = async (channel: typeof channels[0]) => {
    try {
      if (currentChannel?.id === channel.id && isPlaying) {
        await stopPlayback();
        setIsPlaying(false);
      } else {
        await playChannel(channel);
        setCurrentChannel(channel);
        setIsPlaying(true);
      }
    } catch (err) {
      console.error('Failed to play channel:', err);
    }
  };

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Better IPTV
          </h1>
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {channels.length} channels
            </span>
          </div>
        </div>
      </div>

      {/* Search Bar */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4">
        <div className="max-w-7xl mx-auto">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search channels..."
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
            />
          </div>
        </div>
      </div>

      {/* Channel List */}
      <div className="flex-1 overflow-y-auto">
        <div className="max-w-7xl mx-auto p-4">
          {filteredChannels.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-gray-500 dark:text-gray-400">
                {searchQuery ? 'No channels found' : 'No channels available'}
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {filteredChannels.map((channel) => (
                <ChannelCard
                  key={channel.id}
                  channel={channel}
                  isPlaying={currentChannel?.id === channel.id && isPlaying}
                  onPlay={() => handlePlayChannel(channel)}
                />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Now Playing Bar */}
      {currentChannel && (
        <div className="bg-blue-600 text-white p-4">
          <div className="max-w-7xl mx-auto flex items-center justify-between">
            <div className="flex items-center gap-4">
              {currentChannel.logo && (
                <img
                  src={currentChannel.logo}
                  alt={currentChannel.name}
                  className="w-12 h-12 rounded object-cover"
                />
              )}
              <div>
                <p className="font-medium">{currentChannel.name}</p>
                <p className="text-sm text-blue-100">
                  {currentChannel.group_name || 'Live TV'}
                </p>
              </div>
            </div>
            <button
              onClick={async () => {
                await stopPlayback();
                setIsPlaying(false);
              }}
              className="bg-white/20 hover:bg-white/30 p-2 rounded-lg transition-colors"
            >
              <Square className="w-5 h-5" />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

interface ChannelCardProps {
  channel: {
    id?: number;
    name: string;
    logo?: string;
    group_name?: string;
    is_favorite: boolean;
  };
  isPlaying: boolean;
  onPlay: () => void;
}

function ChannelCard({ channel, isPlaying, onPlay }: ChannelCardProps) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden hover:shadow-md transition-shadow">
      <div className="relative">
        {channel.logo ? (
          <img
            src={channel.logo}
            alt={channel.name}
            className="w-full h-32 object-cover"
          />
        ) : (
          <div className="w-full h-32 bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
            <span className="text-3xl font-bold text-white">
              {channel.name.charAt(0).toUpperCase()}
            </span>
          </div>
        )}
        {channel.is_favorite && (
          <div className="absolute top-2 right-2 bg-yellow-400 rounded-full p-1">
            <Star className="w-4 h-4 text-white fill-white" />
          </div>
        )}
      </div>
      <div className="p-3">
        <h3 className="font-medium text-gray-900 dark:text-white truncate">
          {channel.name}
        </h3>
        {channel.group_name && (
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {channel.group_name}
          </p>
        )}
        <button
          onClick={onPlay}
          className={`mt-3 w-full py-2 px-4 rounded-md font-medium transition-colors flex items-center justify-center gap-2 ${
            isPlaying
              ? 'bg-red-600 hover:bg-red-700 text-white'
              : 'bg-blue-600 hover:bg-blue-700 text-white'
          }`}
        >
          {isPlaying ? (
            <>
              <Square className="w-4 h-4" />
              Stop
            </>
          ) : (
            <>
              <Play className="w-4 h-4" />
              Play
            </>
          )}
        </button>
      </div>
    </div>
  );
}
