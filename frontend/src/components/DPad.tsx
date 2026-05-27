import type { FunctionalComponent } from 'preact';
import { useCallback } from 'preact/hooks';
import { Button } from '@fluentui/react-components';
import { 
  ArrowUpRegular, 
  ArrowDownRegular, 
  ArrowLeftRegular, 
  ArrowRightRegular, 
  CircleRegular 
} from '@fluentui/react-icons';

interface DPadProps {
  onDirection: (direction: 'up' | 'down' | 'left' | 'right' | 'center') => void;
}

export const DPad: FunctionalComponent<DPadProps> = ({ onDirection }) => {
  const handleInteraction = useCallback((e: any, direction: 'up' | 'down' | 'left' | 'right' | 'center') => {
    if (e && e.preventDefault) {
      e.preventDefault();
    }
    onDirection(direction);
  }, [onDirection]);

  const baseClass = "flex items-center justify-center select-none touch-none transition-transform active:scale-95";

  return (
    <div className="grid grid-cols-3 grid-rows-3 gap-1 p-2 touch-none select-none">
      <div className="col-start-2">
        <Button
          icon={<ArrowUpRegular style={{ fontSize: '24px' }} />}
          className={`${baseClass} w-12 h-12 md:w-14 md:h-14 rounded-t-lg`}
          appearance="subtle"
          onPointerDown={(e) => handleInteraction(e, 'up')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>

      <div className="col-start-1 row-start-2">
        <Button
          icon={<ArrowLeftRegular style={{ fontSize: '24px' }} />}
          className={`${baseClass} w-12 h-12 md:w-14 md:h-14 rounded-l-lg`}
          appearance="subtle"
          onPointerDown={(e) => handleInteraction(e, 'left')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>
      <div className="col-start-2 row-start-2">
        <Button
          icon={<CircleRegular style={{ fontSize: '24px' }} />}
          className={`${baseClass} w-12 h-12 md:w-14 md:h-14 rounded-full`}
          appearance="subtle"
          onPointerDown={(e) => handleInteraction(e, 'center')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>
      <div className="col-start-3 row-start-2">
        <Button
          icon={<ArrowRightRegular style={{ fontSize: '24px' }} />}
          className={`${baseClass} w-12 h-12 md:w-14 md:h-14 rounded-r-lg`}
          appearance="subtle"
          onPointerDown={(e) => handleInteraction(e, 'right')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>

      <div className="col-start-2 row-start-3">
        <Button
          icon={<ArrowDownRegular style={{ fontSize: '24px' }} />}
          className={`${baseClass} w-12 h-12 md:w-14 md:h-14 rounded-b-lg`}
          appearance="subtle"
          onPointerDown={(e) => handleInteraction(e, 'down')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>
    </div>
  );
};
