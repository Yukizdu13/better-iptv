import { useState, useEffect } from 'react';
import { getVodInfo } from '../lib/tauri';
import { ChevronLeft, Play, Star, Clock, Calendar, Users, Film, Heart } from 'lucide-react';
import type { VodInfo } from '../types';
import { logger } from '../lib/logger';
import { usePlayerStore } from '../stores/player-store';

interface VodViewProps {
  vodId: number;
  vodName: string;
  channelId: number;
  serverUrl: string;
  username: string;
  password: string;
  onBack: () => void;
  onPlay: () => void;
}

export default function VodView({
  vodId,
  vodName,
  channelId,
  serverUrl,
  username,
  password,
  onBack,
  onPlay,
}: VodViewProps) {
  const isFavorite = usePlayerStore((s) => s.channels.find((c) => c.id === channelId)?.is_favorite ?? false);
  const toggleChannelFavorite = usePlayerStore((s) => s.toggleChannelFavorite);
  const [info, setInfo] = useState<VodInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');
  const [plotExpanded, setPlotExpanded] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        setIsLoading(true);
        const data = await getVodInfo(serverUrl, username, password, vodId);
        setInfo(data);
      } catch (err) {
        logger.error('Failed to load VOD info:', err);
        setError(err instanceof Error ? err.message : 'Failed to load movie info');
      } finally {
        setIsLoading(false);
      }
    }
    load();
  }, [vodId, serverUrl, username, password]);

  if (isLoading) {
    return (
      <div className="flex h-screen flex-col bg-gray-50 dark:bg-gray-900">
        <div className="flex flex-1 items-center justify-center">
          <div className="text-center">
            <div className="mx-auto mb-4 h-16 w-16 animate-spin rounded-full border-4 border-blue-500 border-t-transparent" />
            <p className="font-medium text-gray-700 dark:text-gray-300">Chargement…</p>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-screen flex-col bg-gray-50 dark:bg-gray-900">
        <div className="flex flex-1 items-center justify-center">
          <div className="text-center">
            <p className="mb-4 font-medium text-red-600 dark:text-red-400">{error}</p>
            <div className="flex gap-3 justify-center">
              <button
                onClick={onBack}
                className="rounded-md bg-gray-600 px-4 py-2 text-white hover:bg-gray-700"
              >
                Retour
              </button>
              <button
                onClick={onPlay}
                className="flex items-center gap-2 rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
              >
                <Play className="h-4 w-4" />
                Lire quand même
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  const meta = info?.info;
  const posterUrl = meta?.movie_image ?? meta?.backdrop_path?.[0];
  const backdropUrl = meta?.backdrop_path?.[0] ?? meta?.movie_image;
  const title = meta?.name ?? vodName;
  const year = meta?.release_date?.slice(0, 4);

  return (
    <div className="flex h-screen flex-col bg-gray-50 dark:bg-gray-900 overflow-y-auto">
      {/* Backdrop */}
      {backdropUrl && (
        <div className="relative h-56 w-full flex-shrink-0 overflow-hidden">
          <img
            src={backdropUrl}
            alt={title}
            className="h-full w-full object-cover"
          />
          <div className="absolute inset-0 bg-gradient-to-b from-black/20 to-gray-900 dark:to-gray-900" />
        </div>
      )}

      {/* Header bar */}
      <div className={`${backdropUrl ? '-mt-16 relative z-10' : 'border-b border-gray-200 dark:border-gray-700'} bg-white/95 dark:bg-gray-800/95 backdrop-blur px-6 pt-4 pb-6`}>
        <button
          onClick={onBack}
          className="mb-4 flex items-center gap-2 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
        >
          <ChevronLeft className="h-5 w-5" />
          Retour aux films
        </button>

        <div className="flex gap-6">
          {/* Poster */}
          {posterUrl && (
            <img
              src={posterUrl}
              alt={title}
              className="h-48 w-32 flex-shrink-0 rounded-lg object-cover shadow-lg"
            />
          )}

          {/* Info */}
          <div className="flex min-w-0 flex-1 flex-col">
            <h1 className="mb-3 text-2xl font-bold text-gray-900 dark:text-white leading-tight">
              {title}
            </h1>

            {/* Metadata pills */}
            <div className="mb-3 flex flex-wrap gap-3 text-sm">
              {meta?.rating && !isNaN(parseFloat(meta.rating)) && (
                <span className="flex items-center gap-1 font-medium text-yellow-500">
                  <Star className="h-4 w-4 fill-yellow-500" />
                  {parseFloat(meta.rating).toFixed(1)}
                </span>
              )}
              {year && (
                <span className="flex items-center gap-1 text-gray-500 dark:text-gray-400">
                  <Calendar className="h-4 w-4" />
                  {year}
                </span>
              )}
              {meta?.duration && (
                <span className="flex items-center gap-1 text-gray-500 dark:text-gray-400">
                  <Clock className="h-4 w-4" />
                  {meta.duration}
                </span>
              )}
              {meta?.genre && (
                <span className="flex items-center gap-1 text-gray-500 dark:text-gray-400">
                  <Film className="h-4 w-4" />
                  {meta.genre}
                </span>
              )}
            </div>

            {/* Director */}
            {meta?.director && (
              <p className="mb-1 text-sm text-gray-600 dark:text-gray-400">
                <span className="font-medium">Réalisateur :</span> {meta.director}
              </p>
            )}

            {/* Cast */}
            {meta?.cast && (
              <p className="mb-3 text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
                <span className="font-medium">
                  <Users className="mb-0.5 mr-1 inline h-3.5 w-3.5" />
                  Casting :
                </span>{' '}
                {meta.cast}
              </p>
            )}

            {/* Play + Favorite buttons */}
            <div className="mt-auto flex items-center gap-3 pt-2">
              <button
                onClick={onPlay}
                className="flex items-center gap-2 rounded-md bg-blue-600 px-6 py-2.5 font-medium text-white transition-colors hover:bg-blue-700"
              >
                <Play className="h-5 w-5" />
                Lire le film
              </button>
              <button
                onClick={() => toggleChannelFavorite(channelId)}
                aria-label={isFavorite ? 'Retirer des favoris' : 'Ajouter aux favoris'}
                className={`flex items-center gap-2 rounded-md border px-4 py-2.5 font-medium transition-colors ${
                  isFavorite
                    ? 'border-red-500 bg-red-500 text-white hover:bg-red-600'
                    : 'border-gray-300 bg-white text-gray-700 hover:border-red-400 hover:text-red-500 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-300 dark:hover:border-red-400 dark:hover:text-red-400'
                }`}
              >
                <Heart className={`h-5 w-5 ${isFavorite ? 'fill-white' : ''}`} />
                {isFavorite ? 'Favori' : 'Favoris'}
              </button>
            </div>
          </div>
        </div>

        {/* Synopsis */}
        {meta?.plot && (
          <div className="mt-4">
            <h2 className="mb-1 font-semibold text-gray-900 dark:text-white">Synopsis</h2>
            <p className={`text-sm text-gray-700 dark:text-gray-300 ${plotExpanded ? '' : 'line-clamp-4'}`}>
              {meta.plot}
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
  );
}
