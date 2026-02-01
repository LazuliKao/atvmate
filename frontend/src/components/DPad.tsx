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

  const baseButtonClass = "flex items-center justify-center bg-gray-800 hover:bg-gray-700 active:bg-gray-600 text-gray-200 select-none touch-none transition-colors duration-100 shadow-md active:shadow-inner active:scale-95 border border-gray-700";

  return (
    <div className="grid grid-cols-3 grid-rows-3 gap-2 md:gap-3 p-3 md:p-4 lg:p-5 bg-gray-900/80 rounded-3xl backdrop-blur-xl border border-gray-800 shadow-2xl touch-none select-none">
      <div className="col-start-2">
        <Button 
          icon={<ArrowUpRegular style={{ fontSize: '32px' }} />} 
          className={`${baseButtonClass} w-full h-16 md:h-20 lg:h-24 min-w-[64px] md:min-w-[80px] lg:min-w-[96px] rounded-t-2xl rounded-b-md`}
          onPointerDown={(e) => handleInteraction(e, 'up')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>

      <div className="col-start-1 row-start-2">
        <Button 
          icon={<ArrowLeftRegular style={{ fontSize: '32px' }} />} 
          className={`${baseButtonClass} w-16 md:w-20 lg:w-24 h-full min-h-[64px] md:min-h-[80px] lg:min-h-[96px] rounded-l-2xl rounded-r-md`}
          onPointerDown={(e) => handleInteraction(e, 'left')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>
      <div className="col-start-2 row-start-2">
        <Button 
          icon={<CircleRegular style={{ fontSize: '32px' }} />} 
          className={`${baseButtonClass} w-full h-full min-w-[64px] md:min-w-[80px] lg:min-w-[96px] min-h-[64px] md:min-h-[80px] lg:min-h-[96px] rounded-full bg-gray-700 border-gray-600 shadow-inner`}
          onPointerDown={(e) => handleInteraction(e, 'center')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>
      <div className="col-start-3 row-start-2">
        <Button 
          icon={<ArrowRightRegular style={{ fontSize: '32px' }} />} 
          className={`${baseButtonClass} w-16 md:w-20 lg:w-24 h-full min-h-[64px] md:min-h-[80px] lg:min-h-[96px] rounded-r-2xl rounded-l-md`}
          onPointerDown={(e) => handleInteraction(e, 'right')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>

      <div className="col-start-2 row-start-3">
        <Button 
          icon={<ArrowDownRegular style={{ fontSize: '32px' }} />} 
          className={`${baseButtonClass} w-full h-16 md:h-20 lg:h-24 min-w-[64px] md:min-w-[80px] lg:min-w-[96px] rounded-b-2xl rounded-t-md`}
          onPointerDown={(e) => handleInteraction(e, 'down')}
          onContextMenu={(e) => e.preventDefault()}
          style={{ touchAction: 'none' }}
        />
      </div>
    </div>
  );
};
