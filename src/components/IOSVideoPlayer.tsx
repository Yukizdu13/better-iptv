import { useEffect, useRef, useState } from 'react';
import { X, ChevronLeft, ChevronRight } from 'lucide-react';
import type { Channel } from '../types';

interface IOSVideoPlayerProps {
  channel: Channel;
  streamUrls: string[];
  currentProgram?: string | null;
  onClose: () => void;
}

export function IOSVideoPlayer({ channel, streamUrls, currentProgram, onClose }: IOSVideoPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [currentIndex, setCurrentIndex] = useState(0);
  const urls = streamUrls.length > 0 ? streamUrls : [channel.url];

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    video.src = urls[currentIndex];
    video.load();
    video.play().catch(() => {});
  }, [currentIndex, urls]);

  const goTo = (index: number) => {
    if (index >= 0 && index < urls.length) setCurrentIndex(index);
  };

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-black">
      {/* Top bar */}
      <div className="flex items-center gap-3 p-4 text-white">
        <button
          onClick={onClose}
          className="flex-shrink-0 rounded-full bg-white/20 p-2 active:bg-white/40"
          aria-label="Fermer le lecteur"
        >
          <X className="h-5 w-5" />
        </button>

        <div className="min-w-0 flex-1 text-center">
          <p className="truncate font-medium">{channel.name}</p>
          {currentProgram && (
            <p className="truncate text-xs text-gray-300">{currentProgram}</p>
          )}
          {urls.length > 1 && (
            <p className="text-xs text-gray-400">
              Épisode {currentIndex + 1} / {urls.length}
            </p>
          )}
        </div>

        {/* Episode navigation */}
        {urls.length > 1 && (
          <div className="flex flex-shrink-0 items-center gap-1">
            <button
              onClick={() => goTo(currentIndex - 1)}
              disabled={currentIndex === 0}
              className="rounded-full bg-white/20 p-2 disabled:opacity-30 active:bg-white/40"
              aria-label="Épisode précédent"
            >
              <ChevronLeft className="h-5 w-5" />
            </button>
            <button
              onClick={() => goTo(currentIndex + 1)}
              disabled={currentIndex === urls.length - 1}
              className="rounded-full bg-white/20 p-2 disabled:opacity-30 active:bg-white/40"
              aria-label="Épisode suivant"
            >
              <ChevronRight className="h-5 w-5" />
            </button>
          </div>
        )}
      </div>

      {/* Video */}
      <video
        ref={videoRef}
        className="w-full flex-1"
        controls
        playsInline
        autoPlay
        onEnded={() => goTo(currentIndex + 1)}
      />
    </div>
  );
}

export default IOSVideoPlayer;
