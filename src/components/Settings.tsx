import { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import { getSetting, setSetting, fetchEpgData } from '../lib/tauri';
import { usePlayerStore } from '../stores/player-store';
import { logger } from '../lib/logger';
import ProfileManager from './ProfileManager';

interface SettingsProps {
  onClose: () => void;
}

// Language options with ISO codes for MPV
const LANGUAGE_OPTIONS = [
  { code: 'none', name: 'None (Original)', iso: '' },
  { code: 'sv', name: 'Svenska (Swedish)', iso: 'sv,swe,se' },
  { code: 'en', name: 'English', iso: 'en,eng' },
  { code: 'no', name: 'Norsk (Norwegian)', iso: 'no,nor,nb,nn' },
  { code: 'da', name: 'Dansk (Danish)', iso: 'da,dan' },
  { code: 'fi', name: 'Suomi (Finnish)', iso: 'fi,fin' },
  { code: 'de', name: 'Deutsch (German)', iso: 'de,deu,ger' },
  { code: 'fr', name: 'Français (French)', iso: 'fr,fra,fre' },
  { code: 'es', name: 'Español (Spanish)', iso: 'es,spa' },
  { code: 'it', name: 'Italiano (Italian)', iso: 'it,ita' },
  { code: 'pt', name: 'Português (Portuguese)', iso: 'pt,por' },
  { code: 'nl', name: 'Nederlands (Dutch)', iso: 'nl,nld,dut' },
  { code: 'pl', name: 'Polski (Polish)', iso: 'pl,pol' },
  { code: 'ru', name: 'Русский (Russian)', iso: 'ru,rus' },
  { code: 'ar', name: 'العربية (Arabic)', iso: 'ar,ara' },
  { code: 'tr', name: 'Türkçe (Turkish)', iso: 'tr,tur' },
  { code: 'ja', name: '日本語 (Japanese)', iso: 'ja,jpn' },
  { code: 'zh', name: '中文 (Chinese)', iso: 'zh,chi,zho' },
  { code: 'ko', name: '한국어 (Korean)', iso: 'ko,kor' },
];

export default function Settings({ onClose }: SettingsProps) {
  const { triggerEpgRefresh } = usePlayerStore();
  const [epgUrl, setEpgUrl] = useState('https://iptv-epg.org/files/epg-se.xml.gz');
  const [originalEpgUrl, setOriginalEpgUrl] = useState('https://iptv-epg.org/files/epg-se.xml.gz');
  const [theme, setTheme] = useState<'light' | 'dark' | 'system'>('system');
  const [audioLang, setAudioLang] = useState('none');
  const [subtitleLang, setSubtitleLang] = useState('none');
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  // Load settings on mount
  useEffect(() => {
    async function loadSettings() {
      try {
        const savedEpgUrl = await getSetting('epg_url');
        const savedTheme = await getSetting('theme');
        const savedAudioIso = await getSetting('audio_language');
        const savedSubtitleIso = await getSetting('subtitle_language');

        if (savedEpgUrl) {
          setEpgUrl(savedEpgUrl);
          setOriginalEpgUrl(savedEpgUrl); // Remember original for comparison
        }
        if (savedTheme) setTheme(savedTheme as 'light' | 'dark' | 'system');

        // Convert ISO codes back to language codes for UI
        if (savedAudioIso) {
          const audioLang = LANGUAGE_OPTIONS.find((l) => l.iso === savedAudioIso);
          if (audioLang) setAudioLang(audioLang.code);
        }
        if (savedSubtitleIso) {
          const subtitleLang = LANGUAGE_OPTIONS.find((l) => l.iso === savedSubtitleIso);
          if (subtitleLang) setSubtitleLang(subtitleLang.code);
        }
      } catch (err) {
        logger.error('Failed to load settings:', err);
      } finally {
        setIsLoading(false);
      }
    }

    loadSettings();
  }, []);

  const handleSave = async () => {
    try {
      setIsSaving(true);
      await setSetting('epg_url', epgUrl);
      await setSetting('theme', theme);

      // Save language settings (store ISO codes for MPV)
      const audioIso = LANGUAGE_OPTIONS.find((l) => l.code === audioLang)?.iso || '';
      const subtitleIso = LANGUAGE_OPTIONS.find((l) => l.code === subtitleLang)?.iso || '';
      await setSetting('audio_language', audioIso);
      await setSetting('subtitle_language', subtitleIso);

      // Only fetch EPG data if URL has actually changed
      const epgUrlChanged = epgUrl.trim() !== originalEpgUrl.trim();
      if (epgUrlChanged && epgUrl.trim()) {
        logger.info('EPG URL changed, fetching new data from:', epgUrl);
        const count = await fetchEpgData(epgUrl);
        logger.info(`EPG fetched successfully: ${count} programs`);

        // Trigger refresh of EPG data in channel cards
        triggerEpgRefresh();
      } else if (epgUrlChanged) {
        logger.debug('EPG URL cleared, skipping fetch');
      } else {
        logger.debug('EPG URL unchanged, skipping fetch');
      }

      logger.info('Settings saved successfully');
      onClose();
    } catch (err) {
      logger.error('Failed to save settings:', err);
      alert(`Failed to save settings: ${err}. Please check the EPG URL and try again.`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm">
      <div className="max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-lg bg-white shadow-xl dark:bg-gray-800">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-gray-200 p-6 dark:border-gray-700">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Settings</h2>
          <button
            onClick={onClose}
            className="rounded-lg p-2 transition-colors hover:bg-gray-100 dark:hover:bg-gray-700"
          >
            <X className="h-5 w-5 text-gray-600 dark:text-gray-400" />
          </button>
        </div>

        {/* Content */}
        <div className="space-y-6 p-6">
          {/* Profile Management */}
          <div>
            <ProfileManager onClose={onClose} />
          </div>

          {/* EPG Settings */}
          <div>
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Electronic Program Guide (EPG)
            </h3>
            <div className="space-y-4">
              <div>
                <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                  EPG URL (XMLTV format)
                </label>
                <input
                  type="url"
                  value={epgUrl}
                  onChange={(e) => setEpgUrl(e.target.value)}
                  placeholder="http://example.com/epg.xml"
                  className="w-full rounded-lg border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                />
                <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                  Leave empty to use Xtream API EPG (if available)
                </p>
              </div>
            </div>
          </div>

          {/* Appearance Settings */}
          <div>
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Appearance</h3>
            <div className="space-y-4">
              <div>
                <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Theme
                </label>
                <div className="grid grid-cols-3 gap-3">
                  <button
                    onClick={() => setTheme('light')}
                    className={`rounded-lg border px-4 py-2 ${
                      theme === 'light'
                        ? 'border-blue-600 bg-blue-600 text-white'
                        : 'border-gray-300 bg-white text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white'
                    }`}
                  >
                    Light
                  </button>
                  <button
                    onClick={() => setTheme('dark')}
                    className={`rounded-lg border px-4 py-2 ${
                      theme === 'dark'
                        ? 'border-blue-600 bg-blue-600 text-white'
                        : 'border-gray-300 bg-white text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white'
                    }`}
                  >
                    Dark
                  </button>
                  <button
                    onClick={() => setTheme('system')}
                    className={`rounded-lg border px-4 py-2 ${
                      theme === 'system'
                        ? 'border-blue-600 bg-blue-600 text-white'
                        : 'border-gray-300 bg-white text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white'
                    }`}
                  >
                    System
                  </button>
                </div>
              </div>
            </div>
          </div>

          {/* Playback Settings */}
          <div>
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
                  defaultChecked
                  className="h-4 w-4 rounded text-blue-600 focus:ring-blue-500"
                />
              </div>
            </div>
          </div>

          {/* Language Settings */}
          <div>
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Language Settings
            </h3>
            <div className="space-y-4">
              <div>
                <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Default Audio Language
                </label>
                <select
                  value={audioLang}
                  onChange={(e) => setAudioLang(e.target.value)}
                  className="w-full rounded-lg border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:[color-scheme:dark]"
                >
                  {LANGUAGE_OPTIONS.map((lang) => (
                    <option key={lang.code} value={lang.code}>
                      {lang.name}
                    </option>
                  ))}
                </select>
                <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                  Preferred audio track language (if available in stream)
                </p>
              </div>

              <div>
                <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Default Subtitles Language
                </label>
                <select
                  value={subtitleLang}
                  onChange={(e) => setSubtitleLang(e.target.value)}
                  className="w-full rounded-lg border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:[color-scheme:dark]"
                >
                  {LANGUAGE_OPTIONS.map((lang) => (
                    <option key={lang.code} value={lang.code}>
                      {lang.name}
                    </option>
                  ))}
                </select>
                <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                  Preferred subtitle language (if available in stream)
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 border-t border-gray-200 p-6 dark:border-gray-700">
          <button
            onClick={onClose}
            className="rounded-lg px-4 py-2 text-gray-700 transition-colors hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving || isLoading}
            className="rounded-lg bg-blue-600 px-4 py-2 text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {isSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  );
}
