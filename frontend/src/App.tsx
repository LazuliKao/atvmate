import { useState, useCallback, useEffect } from 'preact/hooks';
import { DeviceManager } from './components/DeviceManager';
import { DPad } from './components/DPad';
import { MediaControls } from './components/MediaControls';
import { usePostDevicesDeviceIdKeyKeyName } from './api/default/default';
import { type Theme, webLightTheme, webDarkTheme, Button, tokens } from '@fluentui/react-components';
import { WeatherMoon24Regular, WeatherSunny24Regular, ArrowLeft24Regular, Home24Regular, Navigation24Regular, Desktop24Regular } from '@fluentui/react-icons';

interface AppProps {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

export function App({ theme, setTheme }: AppProps) {
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);

  const sendKeyMutation = usePostDevicesDeviceIdKeyKeyName();

  const sendKey = useCallback((keyName: string) => {
    if (!selectedDevice) return;
    sendKeyMutation.mutate({
      deviceId: selectedDevice,
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
      className="h-screen w-screen font-sans selection:bg-blue-500/30 overflow-hidden touch-none flex flex-col"
      style={{
        backgroundColor: tokens.colorNeutralBackground1,
        color: tokens.colorNeutralForeground1,
      }}
    >
      <div className="absolute top-3 right-3 z-50">
        <Button
          appearance="subtle"
          icon={theme === webDarkTheme ? <WeatherSunny24Regular /> : <WeatherMoon24Regular />}
          onClick={toggleTheme}
          aria-label="Toggle theme"
          size="small"
        />
      </div>

      <main className="flex-1 flex flex-col items-center w-full relative z-10 h-full overflow-y-auto py-4">
        <div className="w-full max-w-md mx-auto px-4 mb-4">
          <DeviceManager onSelectDevice={setSelectedDevice} />
        </div>

        {selectedDevice ? (
          <div className="w-full flex-1 flex flex-col items-center gap-4 animate-in fade-in duration-300">
            <div
              className="flex items-center gap-2 text-xs font-mono px-3 py-1 rounded-full"
              style={{
                backgroundColor: tokens.colorStatusSuccessBackground1,
                color: tokens.colorStatusSuccessForeground1,
              }}
            >
              <span className="w-2 h-2 rounded-full" style={{ backgroundColor: tokens.colorStatusSuccessForeground1 }} />
              {selectedDevice}
            </div>

            <div className="w-full flex flex-col items-center gap-4 px-4">
              <div className="flex flex-col items-center">
                <DPad onDirection={handleDPadDirection} />
              </div>

              <div className="w-full max-w-sm">
                <MediaControls onAction={handleMediaAction} />
              </div>

              <div className="grid grid-cols-3 gap-2 w-full max-w-sm">
                <Button
                  appearance="subtle"
                  icon={<ArrowLeft24Regular />}
                  onClick={() => sendKey('back')}
                  className="flex flex-col items-center justify-center py-3 rounded-lg"
                  size="small"
                >
                  <span className="text-[10px] uppercase font-medium">Back</span>
                </Button>
                <Button
                  appearance="subtle"
                  icon={<Home24Regular />}
                  onClick={() => sendKey('home')}
                  className="flex flex-col items-center justify-center py-3 rounded-lg"
                  size="small"
                >
                  <span className="text-[10px] uppercase font-medium">Home</span>
                </Button>
                <Button
                  appearance="subtle"
                  icon={<Navigation24Regular />}
                  onClick={() => sendKey('menu')}
                  className="flex flex-col items-center justify-center py-3 rounded-lg"
                  size="small"
                >
                  <span className="text-[10px] uppercase font-medium">Menu</span>
                </Button>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-center px-4 opacity-60">
            <div
              className="w-16 h-16 mb-3 rounded-full flex items-center justify-center"
              style={{ backgroundColor: tokens.colorNeutralBackground3 }}
            >
              <Desktop24Regular className="w-8 h-8" style={{ color: tokens.colorNeutralForeground3 }} />
            </div>
            <h3 className="text-base font-medium" style={{ color: tokens.colorNeutralForeground2 }}>No Device Selected</h3>
            <p className="text-xs mt-1 max-w-[200px]" style={{ color: tokens.colorNeutralForeground3 }}>
              Add a device above to start controlling
            </p>
          </div>
        )}
      </main>
    </div>
  );
}
