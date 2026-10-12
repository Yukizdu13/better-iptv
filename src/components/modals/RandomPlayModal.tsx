import { useEffect, useMemo, useRef, useState } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { X, Shuffle, Search, Trash2, FolderOpen, Save, Plus } from 'lucide-react';
import { getSeriesInfo, getRandomPlayPresets, createRandomPlayPreset, updateRandomPlayPreset, deleteRandomPlayPreset } from '../../lib/tauri';
import { logger } from '../../lib/logger';
import { buildBalancedPlaylist } from '../../lib/randomPlaylist';
import type { Channel, QueuedEpisode, RandomPlayPreset } from '../../types';

interface RandomPlayModalProps {
  isOpen: boolean;
  onClose: () => void;
  seriesChannels: Channel[];
  playlistId: number;
  serverUrl: string;
  username: string;
  password: string;
  onPlay: (
    episodeId: string,
    extension: string,
    title: string,
    remainingEpisodes?: QueuedEpisode[]
  ) => void;
}

/** Max number of episodes queued in the shuffled playlist */
const PLAYLIST_SIZE = 30;

/** Height of one series row in the (virtualized) list */
const ROW_HEIGHT = 37;

/** Per-series network timeout: a slow series is skipped instead of blocking the launch */
const SERIES_TIMEOUT_MS = 8000;

/** Episode lists cached in memory (series catalogs rarely change within a session) */
const EPISODES_CACHE_TTL_MS = 60 * 60 * 1000;
const episodesCache = new Map<string, { at: number; episodes: Omit<QueuedEpisode, 'seriesChannelId'>[] }>();

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`timeout after ${ms}ms`)), ms);
    promise.then(
      (v) => { clearTimeout(timer); resolve(v); },
      (e) => { clearTimeout(timer); reject(e); }
    );
  });
}

function extractSeriesId(url?: string): number {
  const urlParts = url?.split('/');
  const idWithExt = urlParts?.[urlParts.length - 1];
  return idWithExt ? parseInt(idWithExt.replace(/\.\w+$/, ''), 10) : NaN;
}

export default function RandomPlayModal({
  isOpen,
  onClose,
  seriesChannels,
  playlistId,
  serverUrl,
  username,
  password,
  onPlay,
}: RandomPlayModalProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [isLoading, setIsLoading] = useState(false);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [error, setError] = useState('');

  // Saved presets (named series selections), scoped to the active playlist
  const [presets, setPresets] = useState<RandomPlayPreset[]>([]);
  const [loadedPresetId, setLoadedPresetId] = useState<number | null>(null);
  const [presetName, setPresetName] = useState('');
  const [presetError, setPresetError] = useState('');
  const [confirmDeleteId, setConfirmDeleteId] = useState<number | null>(null);

  // Presets are metadata only (id/name/list of channel ids) — the series list itself
  // (`seriesChannels`) is never re-fetched here, it's reused as passed in from the parent.
  useEffect(() => {
    if (!isOpen) return;
    getRandomPlayPresets(playlistId)
      .then(setPresets)
      .catch((err) => logger.error('Failed to load random play presets:', err));
  }, [isOpen, playlistId]);

  const filteredSeries = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    return query ? seriesChannels.filter((c) => c.name.toLowerCase().includes(query)) : seriesChannels;
  }, [seriesChannels, searchQuery]);

  // Providers expose 10k+ series: only the visible rows are rendered, otherwise opening the
  // modal mounts thousands of checkboxes and freezes the UI for several seconds.
  const listRef = useRef<HTMLDivElement>(null);
  const rowVirtualizer = useVirtualizer({
    count: filteredSeries.length,
    getScrollElement: () => listRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 10,
  });

  if (!isOpen) return null;

  function toggleSeries(id: number) {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function resetSelection() {
    setSelectedIds(new Set());
    setLoadedPresetId(null);
    setPresetName('');
    setPresetError('');
  }

  function loadPreset(preset: RandomPlayPreset) {
    // Only keep ids that still exist in the current series list (already in memory)
    const validIds = preset.series_channel_ids.filter((id) => seriesChannels.some((c) => c.id === id));
    setSelectedIds(new Set(validIds));
    setLoadedPresetId(preset.id);
    setPresetName(preset.name);
    setPresetError('');
  }

  async function handleSaveAsNew() {
    const name = presetName.trim();
    if (!name || selectedIds.size === 0) return;
    setPresetError('');
    try {
      const id = await createRandomPlayPreset(playlistId, name, Array.from(selectedIds));
      const updated = await getRandomPlayPresets(playlistId);
      setPresets(updated);
      setLoadedPresetId(id);
    } catch (err) {
      logger.error('Failed to save random play preset:', err);
      setPresetError("Échec de l'enregistrement");
    }
  }

  async function handleUpdatePreset() {
    if (loadedPresetId == null) return;
    const name = presetName.trim();
    if (!name || selectedIds.size === 0) return;
    setPresetError('');
    try {
      await updateRandomPlayPreset(loadedPresetId, name, Array.from(selectedIds));
      const updated = await getRandomPlayPresets(playlistId);
      setPresets(updated);
    } catch (err) {
      logger.error('Failed to update random play preset:', err);
      setPresetError('Échec de la mise à jour');
    }
  }

  async function handleDeletePreset(id: number) {
    try {
      await deleteRandomPlayPreset(id);
      setPresets((prev) => prev.filter((p) => p.id !== id));
      if (loadedPresetId === id) resetSelection();
    } catch (err) {
      logger.error('Failed to delete random play preset:', err);
      setPresetError('Échec de la suppression');
    } finally {
      setConfirmDeleteId(null);
    }
  }

  async function handleLaunch() {
    if (selectedIds.size === 0) return;
    setIsLoading(true);
    setError('');

    try {
      const selectedChannels = seriesChannels.filter((c) => selectedIds.has(c.id));
      let done = 0;
      setProgress({ done, total: selectedChannels.length });
      const episodesPerSeries = await Promise.all(
        selectedChannels.map(async (channel) => {
          const seriesId = extractSeriesId(channel.url);
          const finish = () => setProgress({ done: ++done, total: selectedChannels.length });
          if (isNaN(seriesId)) {
            finish();
            return [] as QueuedEpisode[];
          }
          const cacheKey = `${serverUrl}|${username}|${seriesId}`;
          try {
            let cached = episodesCache.get(cacheKey);
            if (!cached || Date.now() - cached.at > EPISODES_CACHE_TTL_MS) {
              const info = await withTimeout(
                getSeriesInfo(serverUrl, username, password, seriesId),
                SERIES_TIMEOUT_MS
              );
              cached = {
                at: Date.now(),
                episodes: Object.values(info.episodes)
                  .flat()
                  .map((ep) => ({
                    id: ep.id,
                    title: ep.title,
                    extension: ep.container_extension,
                    season: String(ep.season),
                    episodeNum: ep.episode_num,
                  })),
              };
              episodesCache.set(cacheKey, cached);
            }
            return cached.episodes.map(
              (ep): QueuedEpisode => ({ ...ep, seriesChannelId: channel.id })
            );
          } catch (err) {
            logger.error(`Failed to load series info for "${channel.name}":`, err);
            return [] as QueuedEpisode[];
          } finally {
            finish();
          }
        })
      );

      if (episodesPerSeries.every((eps) => eps.length === 0)) {
        setError('Aucun épisode trouvé pour les séries sélectionnées');
        return;
      }

      // Every series gets a fair share of the queue, whatever its number of episodes
      const playlist = buildBalancedPlaylist(episodesPerSeries, PLAYLIST_SIZE);

      const perSeries = new Map<string, number>();
      for (const ep of playlist) {
        const name = seriesChannels.find((c) => c.id === ep.seriesChannelId)?.name ?? '?';
        perSeries.set(name, (perSeries.get(name) ?? 0) + 1);
      }
      logger.info(
        `Random queue (${playlist.length} episodes, ${perSeries.size}/${selectedChannels.length} series): ` +
          [...perSeries].map(([name, n]) => `${name} x${n}`).join(', ')
      );

      onPlay(playlist[0].id, playlist[0].extension, playlist[0].title, playlist);
      onClose();
    } finally {
      setIsLoading(false);
      setProgress(null);
    }
  }

  const hasChanges = loadedPresetId == null || presetName.trim() !== presets.find((p) => p.id === loadedPresetId)?.name
    || JSON.stringify([...selectedIds].sort()) !== JSON.stringify([...(presets.find((p) => p.id === loadedPresetId)?.series_channel_ids ?? [])].sort());

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="flex max-h-[85vh] w-full max-w-lg flex-col rounded-lg bg-white p-6 shadow-xl dark:bg-gray-800">
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-xl font-bold text-gray-900 dark:text-white">
            Play aléatoire multi-séries
          </h3>
          <button
            onClick={onClose}
            className="rounded-lg p-1 hover:bg-gray-100 dark:hover:bg-gray-700"
          >
            <X className="h-5 w-5 text-gray-500 dark:text-gray-400" />
          </button>
        </div>

        {/* Saved presets */}
        {presets.length > 0 && (
          <div className="mb-4">
            <p className="mb-1.5 text-xs font-semibold uppercase text-gray-500 dark:text-gray-400">
              Sélections enregistrées
            </p>
            <div className="flex flex-col gap-1">
              {presets.map((preset) => (
                <div
                  key={preset.id}
                  className={`flex items-center justify-between gap-2 rounded-md border px-2 py-1.5 text-sm ${
                    loadedPresetId === preset.id
                      ? 'border-purple-400 bg-purple-50 dark:border-purple-500 dark:bg-purple-900/20'
                      : 'border-gray-200 dark:border-gray-700'
                  }`}
                >
                  <span className="truncate text-gray-900 dark:text-white">
                    {preset.name}{' '}
                    <span className="text-gray-400">({preset.series_channel_ids.length})</span>
                  </span>
                  {confirmDeleteId === preset.id ? (
                    <div className="flex shrink-0 items-center gap-1">
                      <span className="text-xs text-gray-500 dark:text-gray-400">Supprimer ?</span>
                      <button
                        onClick={() => handleDeletePreset(preset.id)}
                        className="rounded px-1.5 py-0.5 text-xs font-medium text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                      >
                        Oui
                      </button>
                      <button
                        onClick={() => setConfirmDeleteId(null)}
                        className="rounded px-1.5 py-0.5 text-xs font-medium text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700"
                      >
                        Non
                      </button>
                    </div>
                  ) : (
                    <div className="flex shrink-0 items-center gap-1">
                      <button
                        onClick={() => loadPreset(preset)}
                        className="rounded p-1 text-gray-500 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700"
                        aria-label={`Charger ${preset.name}`}
                        title="Charger"
                      >
                        <FolderOpen className="h-4 w-4" />
                      </button>
                      <button
                        onClick={() => setConfirmDeleteId(preset.id)}
                        className="rounded p-1 text-gray-500 hover:bg-red-50 hover:text-red-600 dark:text-gray-400 dark:hover:bg-red-900/20"
                        aria-label={`Supprimer ${preset.name}`}
                        title="Supprimer"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Search */}
        <div className="relative mb-3">
          <Search className="pointer-events-none absolute left-2.5 top-2.5 h-4 w-4 text-gray-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Rechercher une série..."
            className="w-full rounded-md border border-gray-300 py-2 pl-9 pr-3 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
          />
        </div>

        <div
          ref={listRef}
          className="mb-3 min-h-[12rem] flex-1 overflow-y-auto rounded-md border border-gray-200 dark:border-gray-700"
        >
          {filteredSeries.length === 0 ? (
            <p className="p-3 text-sm text-gray-500 dark:text-gray-400">Aucune série trouvée</p>
          ) : (
            <div className="relative w-full" style={{ height: rowVirtualizer.getTotalSize() }}>
              {rowVirtualizer.getVirtualItems().map((row) => {
                const channel = filteredSeries[row.index];
                return (
                  <label
                    key={channel.id}
                    className="absolute left-0 top-0 flex w-full cursor-pointer items-center gap-3 border-b border-gray-100 px-3 py-2 hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-700/50"
                    style={{ height: ROW_HEIGHT, transform: `translateY(${row.start}px)` }}
                  >
                    <input
                      type="checkbox"
                      checked={selectedIds.has(channel.id)}
                      onChange={() => toggleSeries(channel.id)}
                      className="h-4 w-4 rounded border-gray-300"
                    />
                    <span className="truncate text-sm text-gray-900 dark:text-white">
                      {channel.name}
                    </span>
                  </label>
                );
              })}
            </div>
          )}
        </div>

        {/* Save / update selection as a named preset */}
        <div className="mb-3 flex items-center gap-2">
          <input
            type="text"
            value={presetName}
            onChange={(e) => setPresetName(e.target.value)}
            placeholder="Nom de la sélection..."
            className="flex-1 rounded-md border border-gray-300 px-3 py-1.5 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
          />
          {loadedPresetId != null && (
            <button
              onClick={handleUpdatePreset}
              disabled={!presetName.trim() || selectedIds.size === 0 || !hasChanges}
              className="flex items-center gap-1 rounded-md bg-blue-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
              title="Mettre à jour la sélection enregistrée"
            >
              <Save className="h-4 w-4" />
              Mettre à jour
            </button>
          )}
          <button
            onClick={handleSaveAsNew}
            disabled={!presetName.trim() || selectedIds.size === 0}
            className="flex items-center gap-1 rounded-md bg-gray-100 px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-200 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600"
            title="Enregistrer comme nouvelle sélection"
          >
            <Plus className="h-4 w-4" />
            Nouveau
          </button>
        </div>
        {presetError && <p className="mb-2 text-sm text-red-600 dark:text-red-400">{presetError}</p>}
        {error && <p className="mb-2 text-sm text-red-600 dark:text-red-400">{error}</p>}

        <div className="flex justify-end gap-3">
          <button
            onClick={onClose}
            className="rounded-lg bg-gray-500 px-4 py-2 text-white hover:bg-gray-600"
          >
            Annuler
          </button>
          <button
            onClick={handleLaunch}
            disabled={selectedIds.size === 0 || isLoading}
            className="flex items-center gap-2 rounded-lg bg-purple-600 px-4 py-2 text-white hover:bg-purple-700 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Shuffle className="h-4 w-4" />
            {isLoading ? (progress ? `Chargement ${progress.done}/${progress.total}…` : 'Chargement...') : `Lancer (${selectedIds.size})`}
          </button>
        </div>
      </div>
    </div>
  );
}
