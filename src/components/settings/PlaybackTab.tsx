import { useEffect, useState } from 'react';
import { getSetting, setSetting } from '../../lib/tauri';
import { logger } from '../../lib/logger';
import { isIOS } from '../../lib/platform';
import { CREDITS_SKIP_SETTING } from '../../hooks/useSeriesProgress';

interface PlaybackTabProps {
  hardwareAcceleration: boolean;
  onHardwareAccelerationChange: (enabled: boolean) => void;
}

export default function PlaybackTab({
  hardwareAcceleration,
  onHardwareAccelerationChange,
}: PlaybackTabProps) {
  // Credits duration (seconds) — saved immediately, independent of the modal's Save button
  const [creditsSkip, setCreditsSkip] = useState('');

  const [videoQuality, setVideoQuality] = useState('balanced');

  useEffect(() => {
    getSetting('video_quality')
      .then((v) => setVideoQuality(v || 'balanced'))
      .catch((err) => logger.error('Failed to load video quality:', err));
    getSetting(CREDITS_SKIP_SETTING)
      .then((v) => setCreditsSkip(v ?? ''))
      .catch((err) => logger.error('Failed to load credits duration:', err));
  }, []);

  function saveVideoQuality(value: string) {
    setVideoQuality(value);
    setSetting('video_quality', value).catch((err) =>
      logger.error('Failed to save video quality:', err)
    );
  }

  function saveCreditsSkip(value: string) {
    const n = Number(value.trim());
    const normalized = value.trim() === '' || !Number.isFinite(n) || n < 0 ? '' : String(Math.round(n));
    setCreditsSkip(normalized);
    setSetting(CREDITS_SKIP_SETTING, normalized).catch((err) =>
      logger.error('Failed to save credits duration:', err)
    );
  }

  return (
    <div className="space-y-6">
      <section>
        <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Playback</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Hardware Acceleration
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                Use GPU for video decoding (recommended)
              </p>
            </div>
            <input
              type="checkbox"
              checked={hardwareAcceleration}
              onChange={(e) => onHardwareAccelerationChange(e.target.checked)}
              className="h-4 w-4 rounded text-blue-600 focus:ring-blue-500"
            />
          </div>
          {!isIOS() && (
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Qualité vidéo (MPV)
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  Économie = moins de chauffe et de bruit de ventilateur. Appliqué à la prochaine
                  lecture.
                </p>
              </div>
              <select
                value={videoQuality}
                onChange={(e) => saveVideoQuality(e.target.value)}
                className="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:[color-scheme:dark]"
              >
                <option value="economy">Économie</option>
                <option value="balanced">Équilibré</option>
                <option value="high">Haute qualité</option>
              </select>
            </div>
          )}
          {!isIOS() && (
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Sauter le générique de fin (séries)
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  Passe à l&apos;épisode suivant quand il reste ce nombre de secondes. 0 ou vide =
                  désactivé. Réglable aussi par série.
                </p>
              </div>
              <div className="flex items-center gap-1 text-sm text-gray-700 dark:text-gray-300">
                <input
                  type="number"
                  min={0}
                  value={creditsSkip}
                  placeholder="0"
                  onChange={(e) => setCreditsSkip(e.target.value)}
                  onBlur={(e) => saveCreditsSkip(e.target.value)}
                  className="w-20 rounded-md border border-gray-300 bg-white px-2 py-1 text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                />
                s
              </div>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
