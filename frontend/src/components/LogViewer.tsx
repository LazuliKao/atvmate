import type { FunctionalComponent } from 'preact';
import { useState, useEffect, useRef, useCallback } from 'preact/hooks';
import { Button, Card, tokens } from '@fluentui/react-components';
import {
  PlugConnected24Regular,
  PlugDisconnected24Regular,
  DeleteRegular,
} from '@fluentui/react-icons';

interface LogViewerProps {
  deviceId: string;
}

export const LogViewer: FunctionalComponent<LogViewerProps> = ({ deviceId }) => {
  const [logs, setLogs] = useState<string[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const logContainerRef = useRef<HTMLDivElement>(null);
  const maxLogLines = 1000;

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    const container = logContainerRef.current;
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  }, [logs]);

  // Cleanup WebSocket on unmount
  useEffect(() => {
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, []);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/devices/${deviceId}/ws`;

    try {
      const ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        setIsConnected(true);
        setLogs((prev) => [...prev, '--- Connected ---']);
      };

      ws.onmessage = (event) => {
        let line = event.data as string;
        try {
          const payload = JSON.parse(line) as { message?: string };
          line = payload.message ?? line;
        } catch {
          // Keep non-JSON messages visible for backwards-compatible servers.
        }
        setLogs((prev) => {
          const newLogs = [...prev, line];
          // Trim to max lines
          if (newLogs.length > maxLogLines) {
            return newLogs.slice(newLogs.length - maxLogLines);
          }
          return newLogs;
        });
      };

      ws.onclose = () => {
        setIsConnected(false);
        setLogs((prev) => [...prev, '--- Disconnected ---']);
        wsRef.current = null;
      };

      ws.onerror = () => {
        setIsConnected(false);
        setLogs((prev) => [...prev, '--- Connection error ---']);
      };

      wsRef.current = ws;
    } catch {
      setLogs((prev) => [...prev, '--- Failed to connect ---']);
    }
  }, [deviceId]);

  const disconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setIsConnected(false);
  }, []);

  const clearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  return (
    <Card
      className="w-full"
      style={{
        backgroundColor: tokens.colorNeutralBackground2,
        color: tokens.colorNeutralForeground1,
      }}
    >
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <PlugConnected24Regular style={{ color: tokens.colorBrandForeground1 }} />
          <h3
            className="text-sm font-semibold"
            style={{ color: tokens.colorBrandForeground1 }}
          >
            Log Viewer
          </h3>
        </div>
        <div className="flex gap-1">
          <Button
            appearance="subtle"
            size="small"
            icon={<DeleteRegular />}
            onClick={clearLogs}
            disabled={logs.length === 0}
            aria-label="Clear logs"
          />
          {isConnected ? (
            <Button
              appearance="subtle"
              size="small"
              icon={<PlugDisconnected24Regular />}
              onClick={disconnect}
              aria-label="Disconnect"
            >
              Stop
            </Button>
          ) : (
            <Button
              appearance="primary"
              size="small"
              icon={<PlugConnected24Regular />}
              onClick={connect}
              aria-label="Connect"
            >
              Start
            </Button>
          )}
        </div>
      </div>

      <div
        ref={logContainerRef}
        className="rounded-lg p-2 font-mono text-[11px] leading-relaxed overflow-y-auto"
        style={{
          backgroundColor: tokens.colorNeutralBackground1,
          height: '240px',
          color: tokens.colorNeutralForeground1,
        }}
      >
        {logs.length === 0 ? (
          <p
            className="text-xs text-center py-4"
            style={{ color: tokens.colorNeutralForeground2 }}
          >
            {isConnected ? 'Waiting for logs...' : 'Click Start to connect'}
          </p>
        ) : (
          logs.map((line, index) => (
            <div
              key={index}
              className="whitespace-pre-wrap break-all"
              style={{
                color: line.startsWith('---')
                  ? tokens.colorNeutralForeground3
                  : tokens.colorNeutralForeground1,
              }}
            >
              {line}
            </div>
          ))
        )}
      </div>
    </Card>
  );
};
