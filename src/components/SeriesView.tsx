import { useState, useEffect } from 'react';
import { usePlayerStore } from '../stores/player-store';
import { getSeriesInfo } from '../lib/tauri';
import { ChevronLeft, Play } from 'lucide-react';
import type { Episode } from '../types';

interface SeriesViewProps {
  seriesId: number;
  seriesName: string;
  serverUrl: string;
  username: string;
  password: string;
  onBack: () => void;
  onPlayEpisode: (
    episodeId: string,
    extension: string,
    title: string,
    remainingEpisodes?: Array<{ id: string; title: string; extension: string }>
  ) => void;
}

export default function SeriesView({
  seriesId,
  seriesName: _seriesName,
  serverUrl,
  username,
  password,
  onBack,
  onPlayEpisode,
}: SeriesViewProps) {
  const { currentSeries, selectedSeason, setCurrentSeries, setSelectedSeason } = usePlayerStore();
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    async function loadSeriesInfo() {
      try {
        setIsLoading(true);
        const info = await getSeriesInfo(serverUrl, username, password, seriesId);
        setCurrentSeries(info);
        // Auto-select first season
        if (info.seasons.length > 0) {
          setSelectedSeason(info.seasons[0].season_number);
        }
      } catch (err) {
        console.error('Failed to load series info:', err);
        setError(err instanceof Error ? err.message : 'Failed to load series');
      } finally {
        setIsLoading(false);
      }
    }

    loadSeriesInfo();

    return () => {
      setCurrentSeries(null);
      setSelectedSeason(null);
    };
  }, [seriesId, serverUrl, username, password, setCurrentSeries, setSelectedSeason]);

  if (isLoading) {
    return (
      <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
        <div className="flex items-center justify-center flex-1">
          <div className="text-center">
            <div className="animate-spin rounded-full h-16 w-16 border-4 border-blue-500 border-t-transparent mx-auto mb-4"></div>
            <p className="text-gray-700 dark:text-gray-300 font-medium">
              Loading series...
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (error || !currentSeries) {
    return (
      <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
        <div className="flex items-center justify-center flex-1">
          <div className="text-center">
            <p className="text-red-600 dark:text-red-400 font-medium mb-4">
              {error || 'Failed to load series'}
            </p>
            <button
              onClick={onBack}
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md"
            >
              Go Back
            </button>
          </div>
        </div>
      </div>
    );
  }

  const selectedSeasonEpisodes = selectedSeason ? currentSeries.episodes[selectedSeason] || [] : [];

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4">
        <div className="max-w-7xl mx-auto">
          <button
            onClick={onBack}
            className="flex items-center gap-2 text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 mb-4"
          >
            <ChevronLeft className="w-5 h-5" />
            Back to Series List
          </button>
          <div className="flex gap-6">
            {currentSeries.info.cover && (
              <img
                src={currentSeries.info.cover}
                alt={currentSeries.info.name}
                className="w-32 h-48 object-cover rounded-lg"
              />
            )}
            <div className="flex-1">
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                {currentSeries.info.name}
              </h1>
              {currentSeries.info.genre && (
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                  {currentSeries.info.genre}
                </p>
              )}
              {currentSeries.info.plot && (
                <p className="text-gray-700 dark:text-gray-300 line-clamp-3">
                  {currentSeries.info.plot}
                </p>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Season Selector */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4">
          <div className="flex gap-2 overflow-x-auto py-4">
            {currentSeries.seasons.map((season) => (
              <button
                key={season.id}
                onClick={() => setSelectedSeason(season.season_number)}
                className={`px-4 py-2 rounded-md font-medium whitespace-nowrap transition-colors ${
                  selectedSeason === season.season_number
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600'
                }`}
              >
                {season.name} ({season.episode_count})
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Episode List */}
      <div className="flex-1 overflow-y-auto">
        <div className="max-w-7xl mx-auto p-4">
          {selectedSeasonEpisodes.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-gray-500 dark:text-gray-400">
                No episodes available for this season
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {selectedSeasonEpisodes.map((episode, index) => {
                // Get all episodes from current to end of season (for playlist playback)
                const remainingEpisodes = selectedSeasonEpisodes
                  .slice(index) // Start from current episode
                  .sort((a, b) => a.episode_num - b.episode_num) // Sort by episode number
                  .map(ep => ({
                    id: ep.id,
                    title: ep.title,
                    extension: ep.container_extension
                  }));

                return (
                  <EpisodeCard
                    key={episode.id}
                    episode={episode}
                    onPlay={() => onPlayEpisode(
                      episode.id,
                      episode.container_extension,
                      episode.title,
                      remainingEpisodes
                    )}
                  />
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

interface EpisodeCardProps {
  episode: Episode;
  onPlay: () => void;
}

function EpisodeCard({ episode, onPlay }: EpisodeCardProps) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden hover:shadow-md transition-shadow">
      <div className="relative bg-gray-900">
        {episode.info.movie_image ? (
          <img
            src={episode.info.movie_image}
            alt={episode.title}
            className="w-full h-48 object-cover"
          />
        ) : (
          <div className="w-full h-48 bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
            <span className="text-4xl font-bold text-white">E{episode.episode_num}</span>
          </div>
        )}
      </div>
      <div className="p-3">
        <h3 className="font-medium text-gray-900 dark:text-white mb-1 line-clamp-2">
          Episode {episode.episode_num}
        </h3>
        <p className="text-sm text-gray-700 dark:text-gray-300 mb-2 line-clamp-1">
          {episode.title}
        </p>
        {episode.info.plot && (
          <p className="text-xs text-gray-600 dark:text-gray-400 mb-3 line-clamp-2">
            {episode.info.plot}
          </p>
        )}
        <button
          onClick={onPlay}
          className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-md transition-colors flex items-center justify-center gap-2"
        >
          <Play className="w-4 h-4" />
          Play
        </button>
      </div>
    </div>
  );
}
