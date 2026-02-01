import type { FunctionalComponent } from 'preact';
import { useCallback } from 'preact/hooks';
import { Button } from '@fluentui/react-components';
import { 
  PlayRegular, 
  PauseRegular, 
  StopRegular, 
  NextRegular, 
  PreviousRegular, 
  Speaker2Regular, 
  Speaker1Regular 
} from '@fluentui/react-icons';

interface MediaControlsProps {
  onAction: (action: 'play_pause' | 'stop' | 'next' | 'previous' | 'volume_up' | 'volume_down') => void;
  isPlaying?: boolean;
}

export const MediaControls: FunctionalComponent<MediaControlsProps> = ({ onAction, isPlaying = false }) => {
  const handleInteraction = useCallback((e: Event, action: 'play_pause' | 'stop' | 'next' | 'previous' | 'volume_up' | 'volume_down') => {
    e.preventDefault();
    onAction(action);
  }, [onAction]);

  const buttonClass = "min-w-[64px] min-h-[64px] rounded-xl touch-none active:scale-95";

  return (
    <div className="flex flex-col gap-3 md:gap-4 lg:gap-5 p-3 md:p-4 lg:p-5 bg-gray-900/80 rounded-3xl backdrop-blur-xl border border-gray-800 shadow-2xl touch-none select-none">
      <div className="flex flex-row gap-2 md:gap-3 lg:gap-4 items-center justify-center">
        <Button
          appearance="secondary"
          icon={<PreviousRegular className="w-6 h-6" />}
          className={buttonClass}
          onPointerDown={(e) => handleInteraction(e, 'previous')}
          onContextMenu={(e) => e.preventDefault()}
          title="previous"
          style={{ touchAction: 'none' }}
        />
        <Button
          appearance="secondary"
          icon={<StopRegular className="w-6 h-6" />}
          className={buttonClass}
          onPointerDown={(e) => handleInteraction(e, 'stop')}
          onContextMenu={(e) => e.preventDefault()}
          title="stop"
          style={{ touchAction: 'none' }}
        />
        <Button 
          appearance="primary" 
          icon={isPlaying ? <PauseRegular className="w-6 h-6" /> : <PlayRegular className="w-6 h-6" />}
          className={`${buttonClass} shadow-blue-900/20`}
          onPointerDown={(e) => handleInteraction(e, 'play_pause')}
          onContextMenu={(e) => e.preventDefault()}
          title={isPlaying ? "pause" : "play"}
          style={{ touchAction: 'none' }}
        />
        <Button
          appearance="secondary"
          icon={<NextRegular className="w-6 h-6" />}
          className={buttonClass}
          onPointerDown={(e) => handleInteraction(e, 'next')}
          onContextMenu={(e) => e.preventDefault()}
          title="next"
          style={{ touchAction: 'none' }}
        />
      </div>

      <div className="flex flex-row gap-2 md:gap-3 lg:gap-4 items-center justify-center border-t border-gray-800 pt-2 md:pt-3 lg:pt-4">
        <Button
          appearance="secondary"
          icon={<Speaker1Regular className="w-6 h-6" />}
          className={`${buttonClass} w-full h-12 md:h-14 lg:h-16 rounded-2xl bg-gray-800/80`}
          onPointerDown={(e) => handleInteraction(e, 'volume_down')}
          onContextMenu={(e) => e.preventDefault()}
          title="volume down"
          style={{ touchAction: 'none' }}
        />
        <Button
          appearance="secondary"
          icon={<Speaker2Regular className="w-6 h-6" />}
          className={`${buttonClass} w-full h-12 md:h-14 lg:h-16 rounded-2xl bg-gray-800/80`}
          onPointerDown={(e) => handleInteraction(e, 'volume_up')}
          onContextMenu={(e) => e.preventDefault()}
          title="volume up"
          style={{ touchAction: 'none' }}
        />
      </div>
    </div>
  );
};
