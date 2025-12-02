import { Settings as SettingsIcon } from 'lucide-react';

interface ChannelHeaderProps {
  /** Total number of channels */
  channelCount: number;
  /** Callback when settings button is clicked */
  onSettingsClick: () => void;
}

/**
 * Header component for the channel list view
 *
 * Displays:
 * - Application title
 * - Channel count
 * - Settings button
 */
export function ChannelHeader({ channelCount, onSettingsClick }: ChannelHeaderProps) {
  return (
    <div className="border-b border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800">
      <div className="mx-auto flex items-center justify-between px-2">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Better IPTV</h1>
        <div className="flex items-center gap-4">
          <span className="text-sm text-gray-600 dark:text-gray-400">
            {channelCount} channels
          </span>
          <button
            onClick={onSettingsClick}
            className="rounded-lg p-2 transition-colors hover:bg-gray-100 dark:hover:bg-gray-700"
            title="Settings"
            aria-label="Open settings"
          >
            <SettingsIcon className="h-5 w-5 text-gray-600 dark:text-gray-400" />
          </button>
        </div>
      </div>
    </div>
  );
}

export default ChannelHeader;
