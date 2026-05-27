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
  const handleInteraction = useCallback((e: any, action: 'play_pause' | 'stop' | 'next' | 'previous' | 'volume_up' | 'volume_down') => {
    e.preventDefault();
    onAction(action);
  }, [onAction]);

  const buttonClass = "min-w-10 min-h-10 rounded-lg touch-none active:scale-95";

  return (
    <div className="flex flex-col gap-2 p-2 touch-none select-none">
      <div className="flex flex-row gap-2 items-center justify-center">
        <Button
          appearance="subtle"
          icon={<PreviousRegular style={{ fontSize: '20px' }} />}
          className={buttonClass}
          size="small"
          onPointerDown={(e) => handleInteraction(e, 'previous')}
          onContextMenu={(e) => e.preventDefault()}
          title="previous"
          style={{ touchAction: 'none' }}
        />
        <Button
          appearance="subtle"
          icon={<StopRegular style={{ fontSize: '20px' }} />}
          className={buttonClass}
          size="small"
          onPointerDown={(e) => handleInteraction(e, 'stop')}
          onContextMenu={(e) => e.preventDefault()}
          title="stop"
          style={{ touchAction: 'none' }}
        />
        <Button
          appearance="primary"
          icon={isPlaying ? <PauseRegular style={{ fontSize: '20px' }} /> : <PlayRegular style={{ fontSize: '20px' }} />}
          className={`${buttonClass} w-12 h-12`}
          size="small"
          onPointerDown={(e) => handleInteraction(e, 'play_pause')}
          onContextMenu={(e) => e.preventDefault()}
          title={isPlaying ? "pause" : "play"}
          style={{ touchAction: 'none' }}
        />
        <Button
          appearance="subtle"
          icon={<NextRegular style={{ fontSize: '20px' }} />}
          className={buttonClass}
          size="small"
          onPointerDown={(e) => handleInteraction(e, 'next')}
          onContextMenu={(e) => e.preventDefault()}
          title="next"
          style={{ touchAction: 'none' }}
        />
      </div>

      <div className="flex flex-row gap-2 items-center justify-center pt-2">
        <Button
          appearance="subtle"
          icon={<Speaker1Regular style={{ fontSize: '20px' }} />}
          className={`${buttonClass} flex-1`}
          size="small"
          onPointerDown={(e) => handleInteraction(e, 'volume_down')}
          onContextMenu={(e) => e.preventDefault()}
          title="volume down"
          style={{ touchAction: 'none' }}
        />
        <Button
          appearance="subtle"
          icon={<Speaker2Regular style={{ fontSize: '20px' }} />}
          className={`${buttonClass} flex-1`}
          size="small"
          onPointerDown={(e) => handleInteraction(e, 'volume_up')}
          onContextMenu={(e) => e.preventDefault()}
          title="volume up"
          style={{ touchAction: 'none' }}
        />
      </div>
    </div>
  );
};
