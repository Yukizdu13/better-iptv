import { useState, useEffect } from 'react';
import { X, Lock } from 'lucide-react';
import { getSetting, setSetting, fetchEpgData, resetParentalPin } from '../lib/tauri';
import { usePlayerStore } from '../stores/player-store';
import { logger } from '../lib/logger';
import ProfileManager from './ProfileManager';
import PinEntryModal from './modals/PinEntryModal';
import ChannelBlockingModal from './modals/ChannelBlockingModal';
import ConfirmationModal from './modals/ConfirmationModal';
import ErrorModal from './modals/ErrorModal';

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
  const { triggerEpgRefresh, channels, loadParentalSettings } = usePlayerStore();
  const [epgUrl, setEpgUrl] = useState('');
  const [originalEpgUrl, setOriginalEpgUrl] = useState('');
  const [theme, setTheme] = useState<'light' | 'dark' | 'system'>('system');
  const [audioLang, setAudioLang] = useState('none');
  const [subtitleLang, setSubtitleLang] = useState('none');
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  // Parental Controls state
  const [parentalEnabled, setParentalEnabled] = useState(false);
  const [hasPin, setHasPin] = useState(false);
  const [blockedChannelIds, setBlockedChannelIds] = useState<Set<number>>(new Set());
  const [blockedCategories, setBlockedCategories] = useState<string[]>([]);
  const [parentalAutoDetect, setParentalAutoDetect] = useState(false);
  const [parentalVisibility, setParentalVisibility] = useState<'hide' | 'lock' | 'blur'>('hide');
  const [showSetPinModal, setShowSetPinModal] = useState(false);
  const [showChangePinModal, setShowChangePinModal] = useState(false);
  const [showResetPinModal, setShowResetPinModal] = useState(false);
  const [showResetPinConfirmation, setShowResetPinConfirmation] = useState(false);
  const [showChannelBlockingModal, setShowChannelBlockingModal] = useState(false);

  // Error modal state
  const [showErrorModal, setShowErrorModal] = useState(false);
  const [errorTitle, setErrorTitle] = useState('');
  const [errorMessage, setErrorMessage] = useState('');

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

        // Load parental controls settings
        const { getParentalSettings, getBlockedChannels } = await import('../lib/tauri');
        const parentalSettings = await getParentalSettings();
        const blockedIds = await getBlockedChannels();

        setParentalEnabled(parentalSettings.enabled);
        setHasPin(parentalSettings.has_pin);
        setBlockedChannelIds(new Set(blockedIds));
        setBlockedCategories(parentalSettings.blocked_categories);
        setParentalAutoDetect(parentalSettings.auto_detect);
        setParentalVisibility(parentalSettings.visibility);
      } catch (err) {
        logger.error('Failed to load settings:', err);
      } finally {
        setIsLoading(false);
      }
    }

    loadSettings();
  }, []);

  // Parental Controls handlers
  const handleResetPin = () => {
    // Show PIN modal to verify before allowing reset
    setShowResetPinModal(true);
  };

  const handleResetPinSuccess = async () => {
    // PIN verified, now show confirmation modal
    setShowResetPinModal(false);
    setShowResetPinConfirmation(true);
  };

  const handleConfirmReset = async () => {
    // User confirmed reset, proceed with resetting PIN
    try {
      await resetParentalPin();
      setHasPin(false);
      setParentalEnabled(false);
      logger.info('Parental PIN reset successfully');
    } catch (err) {
      logger.error('Failed to reset PIN:', err);
      setErrorTitle('Failed to Reset PIN');
      setErrorMessage(`Failed to reset PIN: ${err}`);
      setShowErrorModal(true);
    }
  };

  const handlePinSet = async () => {
    setHasPin(true);
    await loadParentalSettings();
    logger.info('Parental PIN set successfully');
  };

  const handleBlockedChannelsUpdate = (ids: Set<number>) => {
    setBlockedChannelIds(ids);
  };

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

      // Save parental controls settings
      await setSetting('parental_enabled', parentalEnabled.toString());
      await setSetting('parental_auto_detect', parentalAutoDetect.toString());
      await setSetting('parental_visibility', parentalVisibility);

      // If auto-detect is enabled, scan all channels and add adult content to blocked list
      let updatedBlockedIds = new Set(blockedChannelIds);
      if (parentalAutoDetect) {
        const { isAdultContent } = await import('../lib/parentalControls');
        channels.forEach((channel) => {
          if (channel.id && isAdultContent(channel.name, channel.group_name)) {
            updatedBlockedIds.add(channel.id);
          }
        });
        logger.info(
          `Auto-detect found ${updatedBlockedIds.size - blockedChannelIds.size} additional adult channels`
        );
      }

      const { setBlockedChannels } = await import('../lib/tauri');
      await setBlockedChannels(Array.from(updatedBlockedIds));
      await setSetting('parental_blocked_categories', JSON.stringify(blockedCategories));

      // Update local state to reflect the changes
      setBlockedChannelIds(updatedBlockedIds);

      // Reload parental settings to sync with backend
      await loadParentalSettings();

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
      setErrorTitle('Failed to Save Settings');
      setErrorMessage(`Failed to save settings: ${err}. Please check the EPG URL and try again.`);
      setShowErrorModal(true);
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
                  If EPG data is not provided with Xtream, we recommend using:{' '}
                  <a
                    href="https://iptv-epg.org/"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:underline dark:text-blue-400"
                  >
                    https://iptv-epg.org/
                  </a>
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

          {/* Parental Controls */}
          <div>
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Parental Controls
            </h3>
            <div className="space-y-4">
              {/* Enable toggle */}
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Enable Parental Controls
                  </p>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    Restrict access to channels with PIN protection
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={parentalEnabled}
                  onChange={(e) => setParentalEnabled(e.target.checked)}
                  className="h-4 w-4 rounded text-blue-600 focus:ring-blue-500"
                />
              </div>

              {parentalEnabled && (
                <>
                  {/* PIN Setup */}
                  <div>
                    <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                      PIN Code
                    </label>
                    {hasPin ? (
                      <div className="flex gap-2">
                        <button
                          onClick={() => setShowChangePinModal(true)}
                          className="rounded-lg bg-gray-600 px-4 py-2 text-white hover:bg-gray-700"
                        >
                          Change PIN
                        </button>
                        <button
                          onClick={handleResetPin}
                          className="rounded-lg bg-red-600 px-4 py-2 text-white hover:bg-red-700"
                        >
                          Reset PIN
                        </button>
                      </div>
                    ) : (
                      <button
                        onClick={() => setShowSetPinModal(true)}
                        className="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
                      >
                        Set PIN
                      </button>
                    )}
                    <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                      {hasPin ? 'PIN is currently set' : 'No PIN set - parental controls inactive'}
                    </p>
                  </div>

                  {hasPin && (
                    <>
                      {/* Manual Channel Blocking */}
                      <div>
                        <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                          Blocked Channels
                        </label>
                        <button
                          onClick={() => setShowChannelBlockingModal(true)}
                          className="flex items-center gap-2 rounded-lg border border-gray-300 px-4 py-2 hover:bg-gray-50 dark:border-gray-600 dark:hover:bg-gray-700"
                        >
                          <Lock className="h-4 w-4" />
                          <span>Select Channels ({blockedChannelIds.size} blocked)</span>
                        </button>
                      </div>

                      {/* Auto-detection toggle */}
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                            Auto-detect 18+ Content
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            Automatically blocks channels with +18, XXX, Adult in name
                          </p>
                        </div>
                        <input
                          type="checkbox"
                          checked={parentalAutoDetect}
                          onChange={(e) => setParentalAutoDetect(e.target.checked)}
                          className="h-4 w-4 rounded text-blue-600 focus:ring-blue-500"
                        />
                      </div>

                      {/* Visibility mode */}
                      <div>
                        <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                          Visibility Mode
                        </label>
                        <select
                          value={parentalVisibility}
                          onChange={(e) =>
                            setParentalVisibility(e.target.value as 'hide' | 'lock' | 'blur')
                          }
                          className="w-full rounded-lg border border-gray-300 px-4 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:[color-scheme:dark]"
                        >
                          <option value="hide">Hide completely</option>
                          <option value="lock">Show with lock icon</option>
                          <option value="blur">Show blurred</option>
                        </select>
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          How blocked channels appear in the list
                        </p>
                      </div>
                    </>
                  )}
                </>
              )}
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

      {/* Modals */}
      <PinEntryModal
        isOpen={showSetPinModal}
        onClose={() => setShowSetPinModal(false)}
        onSuccess={handlePinSet}
        mode="set"
      />

      <PinEntryModal
        isOpen={showChangePinModal}
        onClose={() => setShowChangePinModal(false)}
        onSuccess={handlePinSet}
        mode="change"
      />

      <PinEntryModal
        isOpen={showResetPinModal}
        onClose={() => setShowResetPinModal(false)}
        onSuccess={handleResetPinSuccess}
        mode="verify"
        title="Enter PIN to reset parental controls"
      />

      <ConfirmationModal
        isOpen={showResetPinConfirmation}
        onClose={() => setShowResetPinConfirmation(false)}
        onConfirm={handleConfirmReset}
        title="Reset PIN?"
        message="Are you sure you want to reset the PIN? This will also disable parental controls."
        confirmText="Reset PIN"
        cancelText="Cancel"
        confirmVariant="danger"
      />

      <ChannelBlockingModal
        isOpen={showChannelBlockingModal}
        onClose={() => setShowChannelBlockingModal(false)}
        channels={channels}
        initialBlockedIds={blockedChannelIds}
        onUpdate={handleBlockedChannelsUpdate}
      />

      <ErrorModal
        isOpen={showErrorModal}
        onClose={() => setShowErrorModal(false)}
        title={errorTitle}
        message={errorMessage}
      />
    </div>
  );
}
