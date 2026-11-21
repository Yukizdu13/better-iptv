import { useMemo, useState } from 'react';
import { X } from 'lucide-react';
import { usePlayerStore, EMPTY_ADVANCED_FILTERS } from '../stores/player-store';
import { extractLanguage, extractYear } from '../hooks/useChannelFilter';
import { setSetting } from '../lib/tauri';
import { logger } from '../lib/logger';

/** Settings key holding the default advanced filters (JSON), applied at startup / tab switch */
export const DEFAULT_FILTERS_SETTING = 'default_advanced_filters';

const RATING_OPTIONS = [5, 6, 7, 7.5, 8, 8.5, 9];

interface FilterPanelProps {
  onClose: () => void;
}

export function FilterPanel({ onClose }: FilterPanelProps) {
  const contentTypeFilter = usePlayerStore((s) => s.contentTypeFilter);
  const advancedFilters = usePlayerStore((s) => s.advancedFilters);
  const setAdvancedFilters = usePlayerStore((s) => s.setAdvancedFilters);
  const defaultAdvancedFilters = usePlayerStore((s) => s.defaultAdvancedFilters);
  const setDefaultAdvancedFilters = usePlayerStore((s) => s.setDefaultAdvancedFilters);
  const [defaultsError, setDefaultsError] = useState('');

  const baseChannels = usePlayerStore((s) =>
    contentTypeFilter === 'vod' ? s.vodChannels : s.seriesChannels
  );

  // Langues disponibles dans les chaînes courantes
  const availableLanguages = useMemo(() => {
    const seen = new Set<string>();
    for (const c of baseChannels) {
      const lang = extractLanguage(c.name);
      if (lang) seen.add(lang);
    }
    return [...seen].sort();
  }, [baseChannels]);

  // Vérifie si des ratings sont disponibles (nécessite un refresh après migration)
  const ratingsAvailable = useMemo(
    () => baseChannels.some((c) => c.rating != null),
    [baseChannels]
  );

  // Plage d'années disponibles
  const yearRange = useMemo(() => {
    let min = Infinity;
    let max = -Infinity;
    for (const c of baseChannels) {
      const y = extractYear(c.name);
      if (y !== null) {
        if (y < min) min = y;
        if (y > max) max = y;
      }
    }
    return min === Infinity ? null : { min, max };
  }, [baseChannels]);

  const { languages, minYear, maxYear, minRating } = advancedFilters;

  function toggleLanguage(lang: string) {
    const next = languages.includes(lang)
      ? languages.filter((l) => l !== lang)
      : [...languages, lang];
    setAdvancedFilters({ ...advancedFilters, languages: next });
  }

  function reset() {
    setAdvancedFilters({ ...EMPTY_ADVANCED_FILTERS });
  }

  const hasActiveFilters =
    languages.length > 0 || minYear !== null || maxYear !== null || minRating !== null;

  const hasDefaults =
    JSON.stringify(defaultAdvancedFilters) !== JSON.stringify(EMPTY_ADVANCED_FILTERS);
  const matchesDefaults =
    JSON.stringify(advancedFilters) === JSON.stringify(defaultAdvancedFilters);

  async function saveDefaults(filters: typeof advancedFilters) {
    setDefaultsError('');
    try {
      await setSetting(DEFAULT_FILTERS_SETTING, JSON.stringify(filters));
      setDefaultAdvancedFilters(filters);
    } catch (err) {
      logger.error('Failed to save default filters:', err);
      setDefaultsError("Échec de l'enregistrement des filtres par défaut");
    }
  }

  return (
    <div className="border-b border-gray-200 bg-white px-4 py-3 dark:border-gray-700 dark:bg-gray-800">
      <div className="mx-auto max-w-7xl">
        <div className="mb-3 flex items-center justify-between">
          <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
            Filtres avancés
          </span>
          <div className="flex items-center gap-3">
            {defaultsError && (
              <span className="text-xs text-red-600 dark:text-red-400">{defaultsError}</span>
            )}
            {hasActiveFilters && !matchesDefaults && (
              <button
                onClick={() => saveDefaults(advancedFilters)}
                className="text-xs text-blue-600 hover:text-blue-700 dark:text-blue-400"
                title="Appliquer automatiquement ces filtres à l'ouverture de l'app"
              >
                Définir par défaut
              </button>
            )}
            {hasDefaults && (
              <button
                onClick={() => saveDefaults({ ...EMPTY_ADVANCED_FILTERS })}
                className="text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                title="Ne plus appliquer de filtres automatiquement"
              >
                Supprimer le défaut
              </button>
            )}
            {hasActiveFilters && (
              <button
                onClick={reset}
                className="text-xs text-blue-600 hover:text-blue-700 dark:text-blue-400"
              >
                Réinitialiser
              </button>
            )}
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        </div>

        <div className="flex flex-wrap gap-6">
          {/* Langue */}
          {availableLanguages.length > 0 && (
            <div className="flex flex-col gap-1.5">
              <span className="text-xs font-medium uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Langue
              </span>
              <div className="flex flex-wrap gap-1.5">
                {availableLanguages.map((lang) => (
                  <button
                    key={lang}
                    onClick={() => toggleLanguage(lang)}
                    className={`rounded-full px-3 py-1 text-xs font-medium transition-colors ${
                      languages.includes(lang)
                        ? 'bg-blue-600 text-white'
                        : 'bg-gray-100 text-gray-700 hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
                    }`}
                  >
                    {lang}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Année */}
          {yearRange && (
            <div className="flex flex-col gap-1.5">
              <span className="text-xs font-medium uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Année
              </span>
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  placeholder={String(yearRange.min)}
                  min={yearRange.min}
                  max={yearRange.max}
                  value={minYear ?? ''}
                  onChange={(e) =>
                    setAdvancedFilters({
                      ...advancedFilters,
                      minYear: e.target.value ? parseInt(e.target.value) : null,
                    })
                  }
                  className="w-20 rounded-md border border-gray-300 bg-white px-2 py-1 text-xs text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                />
                <span className="text-xs text-gray-400">—</span>
                <input
                  type="number"
                  placeholder={String(yearRange.max)}
                  min={yearRange.min}
                  max={yearRange.max}
                  value={maxYear ?? ''}
                  onChange={(e) =>
                    setAdvancedFilters({
                      ...advancedFilters,
                      maxYear: e.target.value ? parseInt(e.target.value) : null,
                    })
                  }
                  className="w-20 rounded-md border border-gray-300 bg-white px-2 py-1 text-xs text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                />
              </div>
            </div>
          )}

          {/* Note minimale */}
          <div className="flex flex-col gap-1.5">
            <span className="text-xs font-medium uppercase tracking-wide text-gray-500 dark:text-gray-400">
              Note min
            </span>
            {ratingsAvailable ? (
              <div className="flex flex-wrap gap-1.5">
                {RATING_OPTIONS.map((r) => (
                  <button
                    key={r}
                    onClick={() =>
                      setAdvancedFilters({
                        ...advancedFilters,
                        minRating: minRating === r ? null : r,
                      })
                    }
                    className={`rounded-full px-3 py-1 text-xs font-medium transition-colors ${
                      minRating === r
                        ? 'bg-yellow-500 text-white'
                        : 'bg-gray-100 text-gray-700 hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
                    }`}
                  >
                    ★ {r}+
                  </button>
                ))}
              </div>
            ) : (
              <p className="text-xs text-amber-600 dark:text-amber-400">
                Faites un <strong>Refresh</strong> de la playlist pour activer ce filtre.
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
