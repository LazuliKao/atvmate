import type { FunctionalComponent } from 'preact';
import { useQuery } from '@tanstack/react-query';
import { Card, tokens } from '@fluentui/react-components';
import { api } from '../api';

interface DeviceStatusPanelProps {
  deviceId: string;
}

export const DeviceStatusPanel: FunctionalComponent<DeviceStatusPanelProps> = ({ deviceId }) => {
  const { data: statuses = [] } = useQuery({
    queryKey: ['deviceStatus'],
    queryFn: api.getStatus,
    refetchInterval: 30_000,
  });
  const status = statuses.find((item) => item.device_id === deviceId);

  return (
    <Card className="w-full" style={{ backgroundColor: tokens.colorNeutralBackground2 }}>
      <div className="flex justify-between text-xs">
        <span style={{ color: tokens.colorNeutralForeground2 }}>Connection</span>
        <span style={{ color: status?.connected ? tokens.colorStatusSuccessForeground1 : tokens.colorStatusDangerForeground1 }}>
          {status?.connected ? `Connected via ${status.device_type}` : 'Offline or unavailable'}
        </span>
      </div>
    </Card>
  );
};
