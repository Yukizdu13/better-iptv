import { Square } from 'lucide-react';
import type { Channel } from '../types';

interface NowPlayingBarProps {
  /** Currently playing channel */
  channel: Channel;
  /** Current EPG program title */
  currentProgram?: string | null;
  /** Next EPG program title */
  nextProgram?: string | null;
  /** Callback when stop button is clicked */
  onStop: () => void;
}

/**
 * Now Playing bar component
 *
 * Displays information about the currently playing channel including:
 * - Channel logo and name
 * - Group/category
 * - Current and next EPG program
 * - Stop button
 */
export function NowPlayingBar({
  channel,
  currentProgram,
  nextProgram,
  onStop,
}: NowPlayingBarProps) {
  return (
    <div className="bg-blue-600 p-4 text-white">
      <div className="mx-auto flex items-center justify-between px-2">
        <div className="flex items-center gap-4">
          {channel.logo && (
            <div className="flex h-12 w-12 items-center justify-center rounded bg-gray-900 p-1">
              <img
                src={channel.logo}
                alt={channel.name}
                className="max-h-full max-w-full object-contain"
              />
            </div>
          )}
          <div>
            <p className="font-medium">{channel.name}</p>
            <p className="text-sm text-blue-100">{channel.group_name || 'Live TV'}</p>
            {currentProgram && (
              <p className="mt-1 text-sm text-blue-200">
                <span className="font-medium">Now showing:</span> {currentProgram}
              </p>
            )}
            {nextProgram && (
              <p className="mt-0.5 text-xs text-blue-200">
                <span className="font-medium">Next up:</span> {nextProgram}
              </p>
            )}
          </div>
        </div>
        <button
          onClick={onStop}
          className="rounded-lg bg-white/20 p-2 transition-colors hover:bg-white/30"
          aria-label="Stop playback"
        >
          <Square className="h-5 w-5" />
        </button>
      </div>
    </div>
  );
}

export default NowPlayingBar;
