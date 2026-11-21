import { useState, useEffect } from 'react';
import { usePlayerStore } from '../stores/player-store';
import {
  getSeriesInfo,
  getLastEpisode,
  setLastEpisode,
  getSetting,
  setSetting,
} from '../lib/tauri';
import { ChevronLeft, Play, Shuffle, Star, Heart, RotateCcw, SkipForward } from 'lucide-react';
import type { Episode, LastEpisode, QueuedEpisode } from '../types';
import { logger } from '../lib/logger';
import { isIOS } from '../lib/platform';
import { CREDITS_SKIP_SETTING, creditsSkipSeriesSetting } from '../hooks/useSeriesProgress';

/** Past this fraction of the duration an episode counts as finished: resume on the next one */
const FINISHED_RATIO = 0.95;
/** Resume a few seconds before the saved position to get back into context */
const RESUME_REWIND_SECONDS = 5;

function formatTime(totalSeconds: number): string {
  const s = Math.floor(totalSeconds);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = String(s % 60).padStart(2, '0');
  return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${m}:${sec}`;
}

interface SeriesViewProps {
  seriesId: number;
  seriesName: string;
  channelId: number;
  serverUrl: string;
  username: string;
  password: string;
  onBack: () => void;
  onPlayEpisode: (
    episodeId: string,
    extension: string,
    title: string,
    remainingEpisodes?: QueuedEpisode[],
    startSeconds?: number
  ) => void;
}

export default function SeriesView({
  seriesId,
  seriesName: _seriesName,
  channelId,
  serverUrl,
  username,
  password,
  onBack,
  onPlayEpisode,
}: SeriesViewProps) {
  const { currentSeries, selectedSeason, setCurrentSeries, setSelectedSeason } = usePlayerStore();
  const isFavorite = usePlayerStore(
    (s) => s.channels.find((c) => c.id === channelId)?.is_favorite ?? false
  );
  const toggleChannelFavorite = usePlayerStore((s) => s.toggleChannelFavorite);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');
  const [plotExpanded, setPlotExpanded] = useState(false);
  const [lastEpisode, setLastEpisodeState] = useState<LastEpisode | null>(null);

  const isPlaying = usePlayerStore((s) => s.isPlaying);
  // Credits duration: per-series override ('' = use the global value)
  const [creditsOverride, setCreditsOverride] = useState('');
  const [globalCredits, setGlobalCredits] = useState('');

  function toQueued(ep: Episode): QueuedEpisode {
    return {
      id: ep.id,
      title: ep.title,
      extension: ep.container_extension,
      seriesChannelId: channelId,
      season: String(ep.season),
      episodeNum: ep.episode_num,
    };
  }

  function recordLastEpisode(episode: Episode) {
    setLastEpisode(
      channelId,
      String(episode.season),
      episode.id,
      episode.episode_num,
      episode.title
    ).catch((err) => logger.error('Failed to save last watched episode:', err));
  }

  function playEpisode(
    episode: Episode,
    remainingEpisodes?: QueuedEpisode[],
    startSeconds?: number
  ) {
    // Saved right away so "Reprendre" works even if playback is stopped early; the exact
    // position is then tracked during playback (useSeriesProgress)
    recordLastEpisode(episode);
    onPlayEpisode(
      episode.id,
      episode.container_extension,
      episode.title,
      remainingEpisodes,
      startSeconds
    );
  }

  function playRandom() {
    if (!currentSeries) return;
    const allEpisodes = Object.values(currentSeries.episodes).flat();
    if (allEpisodes.length === 0) return;
    const shuffled = [...allEpisodes].sort(() => Math.random() - 0.5).slice(0, 20);
    const playlist = shuffled.map(toQueued);
    recordLastEpisode(shuffled[0]);
    onPlayEpisode(playlist[0].id, playlist[0].extension, playlist[0].title, playlist);
  }

  /** Episodes of a season from `episode` to the end, in order */
  function seasonQueueFrom(seasonKey: string, episode: Episode): QueuedEpisode[] {
    return (currentSeries?.episodes[seasonKey] || [])
      .filter((ep) => ep.episode_num >= episode.episode_num)
      .sort((a, b) => a.episode_num - b.episode_num)
      .map(toQueued);
  }

  /** Episode following `episode` (same season, else first episode of the next season) */
  function nextEpisodeAfter(
    seasonKey: string,
    episode: Episode
  ): { season: string; episode: Episode } | null {
    if (!currentSeries) return null;
    const seasonKeys = Object.keys(currentSeries.episodes).sort((a, b) => Number(a) - Number(b));
    const sameSeason = (currentSeries.episodes[seasonKey] || [])
      .filter((ep) => ep.episode_num > episode.episode_num)
      .sort((a, b) => a.episode_num - b.episode_num);
    if (sameSeason.length > 0) return { season: seasonKey, episode: sameSeason[0] };
    const nextKey = seasonKeys[seasonKeys.indexOf(seasonKey) + 1];
    const nextSeason = nextKey
      ? [...(currentSeries.episodes[nextKey] || [])].sort((a, b) => a.episode_num - b.episode_num)
      : [];
    return nextSeason.length > 0 ? { season: nextKey, episode: nextSeason[0] } : null;
  }

  const lastEpisodeFinished =
    lastEpisode?.position_seconds != null &&
    lastEpisode.duration_seconds != null &&
    lastEpisode.duration_seconds > 0 &&
    lastEpisode.position_seconds >= lastEpisode.duration_seconds * FINISHED_RATIO;

  function resumeLastEpisode() {
    if (!lastEpisode || !currentSeries) return;
    const seasonEpisodes = currentSeries.episodes[lastEpisode.season] || [];
    const episode = seasonEpisodes.find((ep) => ep.id === lastEpisode.episode_id);
    if (!episode) return;

    if (lastEpisodeFinished) {
      const next = nextEpisodeAfter(lastEpisode.season, episode);
      if (next) {
        setSelectedSeason(next.season);
        playEpisode(next.episode, seasonQueueFrom(next.season, next.episode));
        return;
      }
    }

    setSelectedSeason(lastEpisode.season);
    const start =
      !lastEpisodeFinished && lastEpisode.position_seconds
        ? Math.max(0, lastEpisode.position_seconds - RESUME_REWIND_SECONDS)
        : undefined;
    playEpisode(episode, seasonQueueFrom(lastEpisode.season, episode), start);
  }

  function resumeLabel(last: LastEpisode): string {
    if (lastEpisodeFinished && currentSeries) {
      const ep = currentSeries.episodes[last.season]?.find((e) => e.id === last.episode_id);
      const next = ep ? nextEpisodeAfter(last.season, ep) : null;
      if (next) return `Épisode suivant S${next.season} E${next.episode.episode_num}`;
    }
    const at =
      last.position_seconds && !lastEpisodeFinished
        ? ` à ${formatTime(last.position_seconds)}`
        : '';
    return `Reprendre S${last.season} E${last.episode_num}${at}`;
  }

  async function saveCreditsOverride(value: string) {
    const trimmed = value.trim();
    const n = Number(trimmed);
    const normalized = trimmed === '' || !Number.isFinite(n) || n < 0 ? '' : String(Math.round(n));
    setCreditsOverride(normalized);
    try {
      await setSetting(creditsSkipSeriesSetting(channelId), normalized);
    } catch (err) {
      logger.error('Failed to save credits duration for series:', err);
    }
  }

  // Load credits settings (global value shown as placeholder)
  useEffect(() => {
    Promise.all([getSetting(creditsSkipSeriesSetting(channelId)), getSetting(CREDITS_SKIP_SETTING)])
      .then(([own, global]) => {
        setCreditsOverride(own ?? '');
        setGlobalCredits(global ?? '');
      })
      .catch((err) => logger.error('Failed to load credits settings:', err));
  }, [channelId]);

  // The position is saved during playback: refresh "Reprendre" once playback stops
  useEffect(() => {
    if (isPlaying) return;
    getLastEpisode(channelId)
      .then(setLastEpisodeState)
      .catch((err) => logger.error('Failed to refresh last watched episode:', err));
  }, [isPlaying, channelId]);

  useEffect(() => {
    async function loadSeriesInfo() {
      try {
        setIsLoading(true);
        const [info, last] = await Promise.all([
          getSeriesInfo(serverUrl, username, password, seriesId),
          getLastEpisode(channelId).catch((err) => {
            logger.error('Failed to load last watched episode:', err);
            return null;
          }),
        ]);
        setCurrentSeries(info);
        setLastEpisodeState(last);
        // Resume on the last watched season, otherwise auto-select the first one
        if (last && info.episodes[last.season]) {
          setSelectedSeason(last.season);
        } else if (info.seasons.length > 0) {
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
  }, [seriesId, channelId, serverUrl, username, password, setCurrentSeries, setSelectedSeason]);

  if (isLoading) {
    return (
      <div className="flex h-full flex-col bg-gray-50 dark:bg-gray-900">
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
      <div className="flex h-full flex-col bg-gray-50 dark:bg-gray-900">
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
    <div className="flex h-full flex-col bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <div className="border-b border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800">
        <div className="mx-auto max-w-7xl">
          <div className="mb-4 flex items-center justify-between">
            <button
              onClick={onBack}
              className="flex items-center gap-2 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
            >
              <ChevronLeft className="h-5 w-5" />
              Retour aux séries
            </button>
            <div className="flex items-center gap-2">
              <button
                onClick={() => toggleChannelFavorite(channelId)}
                aria-label={isFavorite ? 'Retirer des favoris' : 'Ajouter aux favoris'}
                className={`flex items-center gap-2 rounded-md border px-4 py-2 font-medium transition-colors ${
                  isFavorite
                    ? 'border-red-500 bg-red-500 text-white hover:bg-red-600'
                    : 'border-gray-300 bg-white text-gray-700 hover:border-red-400 hover:text-red-500 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-300 dark:hover:border-red-400 dark:hover:text-red-400'
                }`}
              >
                <Heart className={`h-4 w-4 ${isFavorite ? 'fill-white' : ''}`} />
                {isFavorite ? 'Favori' : 'Favoris'}
              </button>
              {lastEpisode && (
                <button
                  onClick={resumeLastEpisode}
                  className="flex items-center gap-2 rounded-md bg-green-600 px-4 py-2 font-medium text-white transition-colors hover:bg-green-700"
                >
                  <RotateCcw className="h-4 w-4" />
                  {resumeLabel(lastEpisode)}
                </button>
              )}
              {!isIOS() && (
                <label
                  className="flex items-center gap-1.5 rounded-md border border-gray-300 px-2 py-1.5 text-sm text-gray-700 dark:border-gray-600 dark:text-gray-300"
                  title="Passer à l'épisode suivant quand il reste ce temps (générique de fin). Vide = réglage global, 0 = désactivé pour cette série."
                >
                  <SkipForward className="h-4 w-4" />
                  Générique
                  <input
                    type="number"
                    min={0}
                    value={creditsOverride}
                    placeholder={globalCredits || '0'}
                    onChange={(e) => setCreditsOverride(e.target.value)}
                    onBlur={(e) => saveCreditsOverride(e.target.value)}
                    className="w-14 rounded border border-gray-300 bg-white px-1 py-0.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                  />
                  s
                </label>
              )}
              <button
                onClick={playRandom}
                className="flex items-center gap-2 rounded-md bg-purple-600 px-4 py-2 font-medium text-white transition-colors hover:bg-purple-700"
              >
                <Shuffle className="h-4 w-4" />
                Play aléatoire
              </button>
            </div>
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
                {currentSeries.info.rating && !isNaN(parseFloat(currentSeries.info.rating)) && (
                  <span className="flex items-center gap-1 text-sm font-medium text-yellow-500">
                    <Star className="h-4 w-4 fill-yellow-500" />
                    {parseFloat(currentSeries.info.rating).toFixed(1)}
                  </span>
                )}
              </div>
              {currentSeries.info.plot && (
                <div>
                  <p
                    className={`text-gray-700 dark:text-gray-300 ${plotExpanded ? '' : 'line-clamp-3'}`}
                  >
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
                  .map(toQueued);

                return (
                  <EpisodeCard
                    key={episode.id}
                    episode={episode}
                    isLastWatched={
                      lastEpisode?.season === selectedSeason &&
                      lastEpisode?.episode_id === episode.id
                    }
                    onPlay={() => playEpisode(episode, remainingEpisodes)}
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
  isLastWatched?: boolean;
  onPlay: () => void;
}

function EpisodeCard({ episode, isLastWatched = false, onPlay }: EpisodeCardProps) {
  const [plotExpanded, setPlotExpanded] = useState(false);

  return (
    <div
      className={`overflow-hidden rounded-lg border bg-white shadow-sm transition-shadow hover:shadow-md dark:bg-gray-800 ${
        isLastWatched
          ? 'border-green-500 dark:border-green-500'
          : 'border-gray-200 dark:border-gray-700'
      }`}
    >
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
            <p
              className={`text-xs text-gray-600 dark:text-gray-400 ${plotExpanded ? '' : 'line-clamp-2'}`}
            >
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
