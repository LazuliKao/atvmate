import type { FunctionalComponent } from 'preact';
import { useCallback } from 'preact/hooks';
import { Play, Pause, Square, SkipForward, SkipBack, Volume2, Volume1 } from 'lucide-preact';

interface MediaControlsProps {
  onAction: (action: 'play_pause' | 'stop' | 'next' | 'previous' | 'volume_up' | 'volume_down') => void;
  isPlaying?: boolean;
}

export const MediaControls: FunctionalComponent<MediaControlsProps> = ({ onAction, isPlaying = false }) => {
  const handleInteraction = useCallback((e: Event, action: 'play_pause' | 'stop' | 'next' | 'previous' | 'volume_up' | 'volume_down') => {
    e.preventDefault();
    onAction(action);
  }, [onAction]);

  const Button = ({ 
    action, 
    icon: Icon, 
    className,
    variant = 'default'
  }: { 
    action: 'play_pause' | 'stop' | 'next' | 'previous' | 'volume_up' | 'volume_down', 
    icon: any, 
    className?: string,
    variant?: 'default' | 'primary'
  }) => (
    <button
      className={`
        flex items-center justify-center 
        rounded-xl select-none touch-none 
        transition-all duration-100 shadow-md active:shadow-inner active:scale-95
        min-w-[64px] min-h-[64px]
        ${variant === 'primary' 
          ? 'bg-blue-600 hover:bg-blue-500 active:bg-blue-700 text-white border border-blue-400/30' 
          : 'bg-gray-800 hover:bg-gray-700 active:bg-gray-600 text-gray-200 border border-gray-700'}
        ${className || ''}
      `}
      onPointerDown={(e) => handleInteraction(e, action)}
      onContextMenu={(e) => e.preventDefault()}
      title={action.replace('_', ' ')}
      style={{ touchAction: 'none' }}
    >
      <Icon size={24} strokeWidth={2.5} />
    </button>
  );

  return (
    <div className="flex flex-col gap-3 md:gap-4 lg:gap-5 p-3 md:p-4 lg:p-5 bg-gray-900/80 rounded-3xl backdrop-blur-xl border border-gray-800 shadow-2xl touch-none select-none">
      <div className="flex flex-row gap-2 md:gap-3 lg:gap-4 items-center justify-center">
        <Button action="previous" icon={SkipBack} />
        <Button action="stop" icon={Square} />
        <Button 
          action="play_pause" 
          icon={isPlaying ? Pause : Play} 
          variant="primary"
          className="shadow-blue-900/20" 
        />
        <Button action="next" icon={SkipForward} />
      </div>

      <div className="flex flex-row gap-2 md:gap-3 lg:gap-4 items-center justify-center border-t border-gray-800 pt-2 md:pt-3 lg:pt-4">
        <Button action="volume_down" icon={Volume1} className="w-full h-12 md:h-14 lg:h-16 rounded-2xl bg-gray-800/80" />
        <Button action="volume_up" icon={Volume2} className="w-full h-12 md:h-14 lg:h-16 rounded-2xl bg-gray-800/80" />
      </div>
    </div>
  );
};
