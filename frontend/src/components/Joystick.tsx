import type { FunctionalComponent } from 'preact';
import { useEffect, useRef, useState } from 'preact/hooks';

interface JoystickProps {
  onMove: (pos: { x: number; y: number }) => void;
  onStop: () => void;
  size?: number;
}

export const Joystick: FunctionalComponent<JoystickProps> = ({ onMove, onStop, size = 150 }) => {
  const baseRef = useRef<HTMLDivElement>(null);
  const stickRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  
  // We track the active touch identifier to handle multitouch correctly
  const touchIdRef = useRef<number | null>(null);

  // Configuration
  const radius = size / 2;
  const stickSize = size / 2.5; 
  const maxDistance = radius;

  useEffect(() => {
    const base = baseRef.current;
    if (!base) return;

    const updatePosition = (clientX: number, clientY: number) => {
      const rect = base.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      const dx = clientX - centerX;
      const dy = clientY - centerY;
      const distance = Math.sqrt(dx * dx + dy * dy);

      const clampedDistance = Math.min(distance, maxDistance);
      
      const angle = Math.atan2(dy, dx);
      const clampedX = Math.cos(angle) * clampedDistance;
      const clampedY = Math.sin(angle) * clampedDistance;

      setPosition({ x: clampedX, y: clampedY });

      // Normalize to -1 to 1
      onMove({
        x: clampedX / maxDistance,
        y: clampedY / maxDistance
      });
    };

    const handleTouchMove = (e: TouchEvent) => {
      // Prevent default to stop scrolling/refresh
      e.preventDefault();
      
      if (touchIdRef.current === null) return;

      const touch = Array.from(e.changedTouches).find(t => t.identifier === touchIdRef.current);
      if (touch) {
        updatePosition(touch.clientX, touch.clientY);
      }
    };

    const handleTouchEnd = (e: TouchEvent) => {
      if (touchIdRef.current === null) return;

      const touch = Array.from(e.changedTouches).find(t => t.identifier === touchIdRef.current);
      if (touch) {
        e.preventDefault();
        
        touchIdRef.current = null;
        setPosition({ x: 0, y: 0 });
        onStop();
        
        window.removeEventListener('touchmove', handleTouchMove);
        window.removeEventListener('touchend', handleTouchEnd);
        window.removeEventListener('touchcancel', handleTouchEnd);
      }
    };

    const handleTouchStart = (e: TouchEvent) => {
      e.preventDefault();
      
      if (touchIdRef.current !== null) return;

      const touch = e.changedTouches[0];
      touchIdRef.current = touch.identifier;

      updatePosition(touch.clientX, touch.clientY);

      window.addEventListener('touchmove', handleTouchMove, { passive: false });
      window.addEventListener('touchend', handleTouchEnd);
      window.addEventListener('touchcancel', handleTouchEnd);
    };

    // Passive: false is crucial for preventing default scroll behavior
    base.addEventListener('touchstart', handleTouchStart, { passive: false });

    return () => {
      base.removeEventListener('touchstart', handleTouchStart);
      window.removeEventListener('touchmove', handleTouchMove);
      window.removeEventListener('touchend', handleTouchEnd);
      window.removeEventListener('touchcancel', handleTouchEnd);
    };
  }, [maxDistance, onMove, onStop]);

  return (
    <div 
      ref={baseRef}
      className="relative rounded-full bg-gray-800/80 border-2 border-gray-600 shadow-xl backdrop-blur-sm touch-none select-none"
      style={{ 
        width: size, 
        height: size,
      }}
    >
      <div className="absolute inset-0 m-auto rounded-full border border-gray-600/30 w-2/3 h-2/3 pointer-events-none" />
      <div className="absolute inset-0 m-auto rounded-full border border-gray-600/30 w-1/3 h-1/3 pointer-events-none" />

      <div 
        ref={stickRef}
        className="absolute rounded-full bg-gradient-to-br from-gray-200 to-gray-400 shadow-2xl pointer-events-none"
        style={{
          width: stickSize,
          height: stickSize,
          top: '50%',
          left: '50%',
          marginTop: -stickSize / 2,
          marginLeft: -stickSize / 2,
          transform: `translate(${position.x}px, ${position.y}px)`,
          transition: touchIdRef.current === null ? 'transform 0.15s ease-out' : 'none'
        }}
      >
        <div className="absolute inset-0 rounded-full bg-white opacity-20 bg-gradient-to-t from-transparent to-white" />
        <div className="absolute inset-[25%] rounded-full bg-gray-300 opacity-50 shadow-inner" />
      </div>
    </div>
  );
};
