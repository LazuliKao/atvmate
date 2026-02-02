import { useState, useCallback, useEffect } from 'preact/hooks';
import { DeviceManager } from './components/DeviceManager';
import { DPad } from './components/DPad';
import { MediaControls } from './components/MediaControls';
import { usePostDevicesIpKeyKeyName } from './api/default/default';
import { type Theme, webLightTheme, webDarkTheme, Button, tokens } from '@fluentui/react-components';
import { WeatherMoon24Regular, WeatherSunny24Regular, ArrowLeft24Regular, Home24Regular, Navigation24Regular, Desktop24Regular } from '@fluentui/react-icons';

interface AppProps {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

export function App({ theme, setTheme }: AppProps) {
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);

  const sendKeyMutation = usePostDevicesIpKeyKeyName();

  const sendKey = useCallback((keyName: string) => {
    if (!selectedDevice) return;
    sendKeyMutation.mutate({
      ip: selectedDevice,
      keyName
    });
  }, [selectedDevice, sendKeyMutation]);

  useEffect(() => {
    const preventDefault = (e: Event) => e.preventDefault();
    document.body.addEventListener('touchmove', preventDefault, { passive: false });
    return () => document.body.removeEventListener('touchmove', preventDefault);
  }, []);

  const handleDPadDirection = useCallback((direction: 'up' | 'down' | 'left' | 'right' | 'center') => {
    const keyMap = {
      'up': 'dpad_up',
      'down': 'dpad_down',
      'left': 'dpad_left',
      'right': 'dpad_right',
      'center': 'dpad_center'
    };
    sendKey(keyMap[direction]);
  }, [sendKey]);

  const handleMediaAction = useCallback((action: 'play_pause' | 'stop' | 'next' | 'previous' | 'volume_up' | 'volume_down') => {
    const keyMap = {
      'play_pause': 'media_play_pause',
      'stop': 'media_stop',
      'next': 'media_next',
      'previous': 'media_previous',
      'volume_up': 'volume_up',
      'volume_down': 'volume_down'
    };
    sendKey(keyMap[action]);
  }, [sendKey]);

  const toggleTheme = useCallback(() => {
    setTheme(theme === webDarkTheme ? webLightTheme : webDarkTheme);
  }, [theme, setTheme]);

  return (
    <div
      className="h-screen w-screen font-sans selection:bg-blue-500/30 overflow-hidden touch-none flex flex-col m-0 p-0"
      style={{
        backgroundColor: tokens.colorNeutralBackground1,
        color: tokens.colorNeutralForeground1,
      }}
    >
      <div className="absolute top-4 right-4 z-50">
        <Button
          appearance="subtle"
          icon={theme === webDarkTheme ? <WeatherSunny24Regular /> : <WeatherMoon24Regular />}
          onClick={toggleTheme}
          aria-label="Toggle theme"
        />
      </div>
      {theme === webDarkTheme && (
        <div className="fixed inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-blue-900/20 via-gray-900/0 to-black pointer-events-none" />
      )}
      
      <main className="flex-1 flex flex-col items-center w-full relative z-10 gap-6 h-full overflow-y-auto">
        <div className="w-full shrink-0 max-w-4xl mx-auto">
          <DeviceManager onSelectDevice={setSelectedDevice} />
        </div>

        {selectedDevice ? (
          <div className="w-full flex-1 flex flex-col items-center justify-start gap-6 md:gap-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
            
            <div
              className="flex items-center gap-2 text-xs font-mono px-3 py-1 rounded-full border"
              style={{
                backgroundColor: tokens.colorStatusSuccessBackground1,
                color: tokens.colorStatusSuccessForeground1,
                borderColor: tokens.colorStatusSuccessBorder1,
              }}
            >
              <span className="w-2 h-2 rounded-full animate-pulse" style={{ backgroundColor: tokens.colorStatusSuccessForeground1 }} />
              CONNECTED: {selectedDevice}
            </div>

            <div className="w-full flex flex-col items-center gap-6 md:gap-8">
              <div className="flex flex-col items-center gap-2 flex-1">
                <DPad onDirection={handleDPadDirection} />
                <span className="text-xs uppercase tracking-widest font-bold" style={{ color: tokens.colorNeutralForeground2 }}>Navigation</span>
              </div>

              <div className="w-full max-w-sm md:max-w-lg mx-auto">
                <MediaControls onAction={handleMediaAction} />
              </div>

              <div className="grid grid-cols-3 gap-3 md:gap-4 w-full max-w-sm md:max-w-lg mx-auto px-2">
                <Button
                  appearance="secondary"
                  icon={<ArrowLeft24Regular />}
                  onClick={() => sendKey('back')}
                  className="flex flex-col items-center justify-center p-2 md:p-3 rounded-xl transition-all"
                >
                  <span className="text-[10px] md:text-xs uppercase font-bold tracking-wider">Back</span>
                </Button>
                <Button
                  appearance="secondary"
                  icon={<Home24Regular />}
                  onClick={() => sendKey('home')}
                  className="flex flex-col items-center justify-center p-2 md:p-3 rounded-xl transition-all"
                >
                  <span className="text-[10px] md:text-xs uppercase font-bold tracking-wider">Home</span>
                </Button>
                <Button
                  appearance="secondary"
                  icon={<Navigation24Regular />}
                  onClick={() => sendKey('menu')}
                  className="flex flex-col items-center justify-center p-2 md:p-3 rounded-xl transition-all"
                >
                  <span className="text-[10px] md:text-xs uppercase font-bold tracking-wider">Menu</span>
                </Button>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex-1 flex flex flex-col items-center justify-center text-center p-4 md:p-8 opacity-50 animate-pulse">
            <div
              className="w-20 h-20 md:w-24 md:h-24 mb-4 rounded-full flex items-center justify-center"
              style={{ backgroundColor: tokens.colorNeutralBackground3 }}
            >
              <Desktop24Regular className="w-12 h-12 md:w-16 md:h-16" style={{ color: tokens.colorNeutralForeground3 }} />
            </div>
            <h3 className="text-lg md:text-xl font-bold" style={{ color: tokens.colorNeutralForeground2 }}>No Device Selected</h3>
            <p className="text-xs md:text-sm mt-2 max-w-[200px] md:max-w-[300px]" style={{ color: tokens.colorNeutralForeground3 }}>Select or add a device from the manager above to start controlling.</p>
          </div>
        )}

      </main>
    </div>
  );
}
