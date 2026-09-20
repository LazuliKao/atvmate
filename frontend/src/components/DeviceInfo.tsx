import type { FunctionalComponent } from 'preact';
import { useQuery } from '@tanstack/react-query';
import { Button, Card, tokens, Spinner } from '@fluentui/react-components';
import { ArrowClockwiseRegular, Phone24Regular } from '@fluentui/react-icons';
import { api } from '../api';

interface DeviceInfoProps {
  deviceId: string;
}

export const DeviceInfo: FunctionalComponent<DeviceInfoProps> = ({ deviceId }) => {
  const { data, isLoading, isError, refetch } = useQuery({
    queryKey: ['deviceInfo', deviceId],
    queryFn: async () => {
      return api.getDeviceInfo(deviceId);
    },
    enabled: !!deviceId,
  });

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
          <Phone24Regular style={{ color: tokens.colorBrandForeground1 }} />
          <h3
            className="text-sm font-semibold"
            style={{ color: tokens.colorBrandForeground1 }}
          >
            Device Info
          </h3>
        </div>
        <Button
          appearance="subtle"
          size="small"
          icon={<ArrowClockwiseRegular />}
          onClick={() => refetch()}
          aria-label="Refresh device info"
        />
      </div>

      {isLoading && (
        <div className="flex items-center justify-center py-4">
          <Spinner size="tiny" />
        </div>
      )}

      {isError && (
        <p
          className="text-xs text-center py-2"
          style={{ color: tokens.colorStatusDangerForeground1 }}
        >
          Failed to load device info
        </p>
      )}

      {data && (
        <div className="space-y-2">
          <div className="flex justify-between items-center py-1">
            <span className="text-xs" style={{ color: tokens.colorNeutralForeground2 }}>
              Model
            </span>
            <span className="text-xs font-mono">{data.model}</span>
          </div>
          <div
            className="border-t"
            style={{ borderColor: tokens.colorNeutralStroke2 }}
          />
          <div className="flex justify-between items-center py-1">
            <span className="text-xs" style={{ color: tokens.colorNeutralForeground2 }}>
              Android
            </span>
            <span className="text-xs font-mono">{data.android_version}</span>
          </div>
          <div
            className="border-t"
            style={{ borderColor: tokens.colorNeutralStroke2 }}
          />
          <div className="flex justify-between items-center py-1">
            <span className="text-xs" style={{ color: tokens.colorNeutralForeground2 }}>
              Serial
            </span>
            <span className="text-xs font-mono">{data.serial}</span>
          </div>
          <div
            className="border-t"
            style={{ borderColor: tokens.colorNeutralStroke2 }}
          />
          <div className="flex justify-between items-center py-1">
            <span className="text-xs" style={{ color: tokens.colorNeutralForeground2 }}>
              Resolution
            </span>
            <span className="text-xs font-mono">{data.screen_resolution}</span>
          </div>
        </div>
      )}
    </Card>
  );
};
