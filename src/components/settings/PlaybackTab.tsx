interface PlaybackTabProps {
  hardwareAcceleration: boolean;
  onHardwareAccelerationChange: (enabled: boolean) => void;
}

export default function PlaybackTab({
  hardwareAcceleration,
  onHardwareAccelerationChange,
}: PlaybackTabProps) {
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
        </div>
      </section>
    </div>
  );
}
