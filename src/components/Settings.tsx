import { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import { getSetting, setSetting, fetchEpgData } from '../lib/tauri';
import { usePlayerStore } from '../stores/player-store';

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

        if (savedEpgUrl) setEpgUrl(savedEpgUrl);
        if (savedTheme) setTheme(savedTheme as 'light' | 'dark' | 'system');

        // Convert ISO codes back to language codes for UI
        if (savedAudioIso) {
          const audioLang = LANGUAGE_OPTIONS.find(l => l.iso === savedAudioIso);
          if (audioLang) setAudioLang(audioLang.code);
        }
        if (savedSubtitleIso) {
          const subtitleLang = LANGUAGE_OPTIONS.find(l => l.iso === savedSubtitleIso);
          if (subtitleLang) setSubtitleLang(subtitleLang.code);
        }
      } catch (err) {
        console.error('Failed to load settings:', err);
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
      const audioIso = LANGUAGE_OPTIONS.find(l => l.code === audioLang)?.iso || '';
      const subtitleIso = LANGUAGE_OPTIONS.find(l => l.code === subtitleLang)?.iso || '';
      await setSetting('audio_language', audioIso);
      await setSetting('subtitle_language', subtitleIso);

      // Fetch EPG data if URL is provided
      if (epgUrl.trim()) {
        console.log('Fetching EPG from:', epgUrl);
        const count = await fetchEpgData(epgUrl);
        console.log(`EPG fetched successfully: ${count} programs`);

        // Trigger refresh of EPG data in channel cards
        triggerEpgRefresh();
      }

      console.log('Settings saved successfully');
      onClose();
    } catch (err) {
      console.error('Failed to save settings:', err);
      alert(`Failed to save settings: ${err}. Please check the EPG URL and try again.`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Settings</h2>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          >
            <X className="w-5 h-5 text-gray-600 dark:text-gray-400" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* EPG Settings */}
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Electronic Program Guide (EPG)
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  EPG URL (XMLTV format)
                </label>
                <input
                  type="url"
                  value={epgUrl}
                  onChange={(e) => setEpgUrl(e.target.value)}
                  placeholder="http://example.com/epg.xml"
                  className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Leave empty to use Xtream API EPG (if available)
                </p>
              </div>
            </div>
          </div>

          {/* Appearance Settings */}
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Appearance
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Theme
                </label>
                <div className="grid grid-cols-3 gap-3">
                  <button
                    onClick={() => setTheme('light')}
                    className={`px-4 py-2 rounded-lg border ${
                      theme === 'light'
                        ? 'bg-blue-600 text-white border-blue-600'
                        : 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white border-gray-300 dark:border-gray-600'
                    }`}
                  >
                    Light
                  </button>
                  <button
                    onClick={() => setTheme('dark')}
                    className={`px-4 py-2 rounded-lg border ${
                      theme === 'dark'
                        ? 'bg-blue-600 text-white border-blue-600'
                        : 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white border-gray-300 dark:border-gray-600'
                    }`}
                  >
                    Dark
                  </button>
                  <button
                    onClick={() => setTheme('system')}
                    className={`px-4 py-2 rounded-lg border ${
                      theme === 'system'
                        ? 'bg-blue-600 text-white border-blue-600'
                        : 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white border-gray-300 dark:border-gray-600'
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
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Playback
            </h3>
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
                  className="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
                />
              </div>
            </div>
          </div>

          {/* Language Settings */}
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Language Settings
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Default Audio Language
                </label>
                <select
                  value={audioLang}
                  onChange={(e) => setAudioLang(e.target.value)}
                  className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
                >
                  {LANGUAGE_OPTIONS.map((lang) => (
                    <option key={lang.code} value={lang.code}>
                      {lang.name}
                    </option>
                  ))}
                </select>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Preferred audio track language (if available in stream)
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Default Subtitles Language
                </label>
                <select
                  value={subtitleLang}
                  onChange={(e) => setSubtitleLang(e.target.value)}
                  className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
                >
                  {LANGUAGE_OPTIONS.map((lang) => (
                    <option key={lang.code} value={lang.code}>
                      {lang.name}
                    </option>
                  ))}
                </select>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Preferred subtitle language (if available in stream)
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-6 border-t border-gray-200 dark:border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving || isLoading}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  );
}
