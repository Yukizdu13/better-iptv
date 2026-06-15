import { useState, useEffect } from 'react';
import { usePlayerStore } from '../stores/player-store';
import { getSeriesInfo } from '../lib/tauri';
import { ChevronLeft, Play, Shuffle, Star } from 'lucide-react';
import type { Episode } from '../types';
import { logger } from '../lib/logger';

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
  const [plotExpanded, setPlotExpanded] = useState(false);

  function playRandom() {
    if (!currentSeries) return;
    const allEpisodes = Object.values(currentSeries.episodes).flat();
    if (allEpisodes.length === 0) return;
    const shuffled = [...allEpisodes].sort(() => Math.random() - 0.5).slice(0, 20);
    const playlist = shuffled.map((ep) => ({
      id: ep.id,
      title: ep.title,
      extension: ep.container_extension,
    }));
    onPlayEpisode(playlist[0].id, playlist[0].extension, playlist[0].title, playlist);
  }

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
        logger.error('Failed to load series info:', err);
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
      <div className="flex h-screen flex-col bg-gray-50 dark:bg-gray-900">
        <div className="flex flex-1 items-center justify-center">
          <div className="text-center">
            <div className="mx-auto mb-4 h-16 w-16 animate-spin rounded-full border-4 border-blue-500 border-t-transparent"></div>
            <p className="font-medium text-gray-700 dark:text-gray-300">Loading series...</p>
          </div>
        </div>
      </div>
    );
  }

  if (error || !currentSeries) {
    return (
      <div className="flex h-screen flex-col bg-gray-50 dark:bg-gray-900">
        <div className="flex flex-1 items-center justify-center">
          <div className="text-center">
            <p className="mb-4 font-medium text-red-600 dark:text-red-400">
              {error || 'Failed to load series'}
            </p>
            <button
              onClick={onBack}
              className="rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
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
    <div className="flex h-screen flex-col bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="border-b border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800">
        <div className="mx-auto max-w-7xl">
          <div className="mb-4 flex items-center justify-between">
            <button
              onClick={onBack}
              className="flex items-center gap-2 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
            >
              <ChevronLeft className="h-5 w-5" />
              Back to Series List
            </button>
            <button
              onClick={playRandom}
              className="flex items-center gap-2 rounded-md bg-purple-600 px-4 py-2 font-medium text-white transition-colors hover:bg-purple-700"
            >
              <Shuffle className="h-4 w-4" />
              Play aléatoire
            </button>
          </div>
          <div className="flex gap-6">
            {(currentSeries.info.cover || currentSeries.info.backdrop_path?.[0]) && (
              <img
                src={currentSeries.info.cover ?? currentSeries.info.backdrop_path![0]}
                alt={currentSeries.info.name}
                className="h-48 w-32 rounded-lg object-cover"
              />
            )}
            <div className="flex-1">
              <h1 className="mb-2 text-3xl font-bold text-gray-900 dark:text-white">
                {currentSeries.info.name}
              </h1>
              <div className="mb-2 flex flex-wrap items-center gap-3">
                {currentSeries.info.genre && (
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    {currentSeries.info.genre}
                  </span>
                )}
                {currentSeries.info.releaseDate && (
                  <span className="text-sm text-gray-500 dark:text-gray-500">
                    {currentSeries.info.releaseDate}
                  </span>
                )}
                {currentSeries.info.rating && (
                  <span className="flex items-center gap-1 text-sm font-medium text-yellow-500">
                    <Star className="h-4 w-4 fill-yellow-500" />
                    {currentSeries.info.rating}
                  </span>
                )}
              </div>
              {currentSeries.info.plot && (
                <div>
                  <p className={`text-gray-700 dark:text-gray-300 ${plotExpanded ? '' : 'line-clamp-3'}`}>
                    {currentSeries.info.plot}
                  </p>
                  <button
                    onClick={() => setPlotExpanded(!plotExpanded)}
                    className="mt-1 text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
                  >
                    {plotExpanded ? 'Voir moins' : 'Voir plus'}
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Season Selector */}
      <div className="border-b border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800">
        <div className="mx-auto max-w-7xl px-4">
          <div className="flex gap-2 overflow-x-auto py-4">
            {currentSeries.seasons.map((season) => (
              <button
                key={season.id}
                onClick={() => setSelectedSeason(season.season_number)}
                className={`whitespace-nowrap rounded-md px-4 py-2 font-medium transition-colors ${
                  selectedSeason === season.season_number
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-200 text-gray-900 hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600'
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
        <div className="mx-auto max-w-7xl p-4">
          {selectedSeasonEpisodes.length === 0 ? (
            <div className="py-12 text-center">
              <p className="text-gray-500 dark:text-gray-400">
                No episodes available for this season
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {selectedSeasonEpisodes.map((episode, index) => {
                // Get all episodes from current to end of season (for playlist playback)
                const remainingEpisodes = selectedSeasonEpisodes
                  .slice(index) // Start from current episode
                  .sort((a, b) => a.episode_num - b.episode_num) // Sort by episode number
                  .map((ep) => ({
                    id: ep.id,
                    title: ep.title,
                    extension: ep.container_extension,
                  }));

                return (
                  <EpisodeCard
                    key={episode.id}
                    episode={episode}
                    onPlay={() =>
                      onPlayEpisode(
                        episode.id,
                        episode.container_extension,
                        episode.title,
                        remainingEpisodes
                      )
                    }
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
  const [plotExpanded, setPlotExpanded] = useState(false);

  return (
    <div className="overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm transition-shadow hover:shadow-md dark:border-gray-700 dark:bg-gray-800">
      <div className="relative bg-gray-900">
        {episode.info.movie_image ? (
          <img
            src={episode.info.movie_image}
            alt={episode.title}
            className="h-48 w-full object-cover"
          />
        ) : (
          <div className="flex h-48 w-full items-center justify-center bg-gradient-to-br from-blue-500 to-purple-600">
            <span className="text-4xl font-bold text-white">E{episode.episode_num}</span>
          </div>
        )}
      </div>
      <div className="p-3">
        <div className="mb-1 flex items-start justify-between gap-2">
          <h3 className="line-clamp-2 font-medium text-gray-900 dark:text-white">
            Episode {episode.episode_num}
          </h3>
          {episode.info.rating != null && (
            <span className="flex shrink-0 items-center gap-0.5 text-xs font-medium text-yellow-500">
              <Star className="h-3 w-3 fill-yellow-500" />
              {episode.info.rating.toFixed(1)}
            </span>
          )}
        </div>
        <p className="mb-2 line-clamp-1 text-sm text-gray-700 dark:text-gray-300">
          {episode.title}
        </p>
        {episode.info.plot && (
          <div className="mb-3">
            <p className={`text-xs text-gray-600 dark:text-gray-400 ${plotExpanded ? '' : 'line-clamp-2'}`}>
              {episode.info.plot}
            </p>
            <button
              onClick={() => setPlotExpanded(!plotExpanded)}
              className="mt-0.5 text-xs text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
            >
              {plotExpanded ? 'Voir moins' : 'Voir plus'}
            </button>
          </div>
        )}
        <button
          onClick={onPlay}
          className="flex w-full items-center justify-center gap-2 rounded-md bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
        >
          <Play className="h-4 w-4" />
          Play
        </button>
      </div>
    </div>
  );
}
