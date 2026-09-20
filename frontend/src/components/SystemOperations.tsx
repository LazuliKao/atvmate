import type { FunctionalComponent } from 'preact';
import { useState, useCallback } from 'preact/hooks';
import { useMutation } from '@tanstack/react-query';
import { Button, Card, Input, tokens } from '@fluentui/react-components';
import {
  Power24Regular,
  ArrowReset24Regular,
  ShieldLock24Regular,
  BroomRegular,
  Info24Regular,
} from '@fluentui/react-icons';
import { api } from '../api';

interface SystemOperationsProps {
  deviceId: string;
}

export const SystemOperations: FunctionalComponent<SystemOperationsProps> = ({ deviceId }) => {
  const [clearPackage, setClearPackage] = useState('');
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const rebootMutation = useMutation({
    mutationFn: () => api.reboot(deviceId),
    onSuccess: () => setStatusMessage('Rebooting...'),
    onError: () => setStatusMessage('Reboot failed'),
  });

  const recoveryMutation = useMutation({
    mutationFn: () => api.reboot(deviceId, '/recovery'),
    onSuccess: () => setStatusMessage('Rebooting to recovery...'),
    onError: () => setStatusMessage('Reboot to recovery failed'),
  });

  const bootloaderMutation = useMutation({
    mutationFn: () => api.reboot(deviceId, '/bootloader'),
    onSuccess: () => setStatusMessage('Rebooting to bootloader...'),
    onError: () => setStatusMessage('Reboot to bootloader failed'),
  });

  const clearDataMutation = useMutation({
    mutationFn: (packageName: string) => api.clearAppData(deviceId, packageName),
    onSuccess: () => {
      setStatusMessage(`Cleared data for ${clearPackage}`);
      setClearPackage('');
    },
    onError: () => setStatusMessage('Clear data failed'),
  });

  const handleClearData = useCallback(() => {
    if (clearPackage.trim()) {
      clearDataMutation.mutate(clearPackage.trim());
    }
  }, [clearPackage, clearDataMutation]);

  const isAnyPending =
    rebootMutation.isPending ||
    recoveryMutation.isPending ||
    bootloaderMutation.isPending ||
    clearDataMutation.isPending;

  return (
    <Card
      className="w-full"
      style={{
        backgroundColor: tokens.colorNeutralBackground2,
        color: tokens.colorNeutralForeground1,
      }}
    >
      <div className="flex items-center gap-2 mb-3">
        <Power24Regular style={{ color: tokens.colorBrandForeground1 }} />
        <h3
          className="text-sm font-semibold"
          style={{ color: tokens.colorBrandForeground1 }}
        >
          System Operations
        </h3>
      </div>

      <div className="grid grid-cols-3 gap-2 mb-3">
        <Button
          appearance="outline"
          size="small"
          icon={<ArrowReset24Regular />}
          onClick={() => {
            if (confirm('Reboot device?')) rebootMutation.mutate();
          }}
          disabled={isAnyPending}
          className="flex flex-col items-center justify-center py-3"
        >
          <span className="text-[10px] uppercase font-medium mt-1">Reboot</span>
        </Button>
        <Button
          appearance="outline"
          size="small"
          icon={<ShieldLock24Regular />}
          onClick={() => {
            if (confirm('Reboot to recovery mode?')) recoveryMutation.mutate();
          }}
          disabled={isAnyPending}
          className="flex flex-col items-center justify-center py-3"
        >
          <span className="text-[10px] uppercase font-medium mt-1">Recovery</span>
        </Button>
        <Button
          appearance="outline"
          size="small"
          icon={<Info24Regular />}
          onClick={() => {
            if (confirm('Reboot to bootloader?')) bootloaderMutation.mutate();
          }}
          disabled={isAnyPending}
          className="flex flex-col items-center justify-center py-3"
        >
          <span className="text-[10px] uppercase font-medium mt-1">Bootloader</span>
        </Button>
      </div>

      <div
        className="border-t pt-3"
        style={{ borderColor: tokens.colorNeutralStroke2 }}
      >
        <p
          className="text-xs font-medium mb-2"
          style={{ color: tokens.colorNeutralForeground2 }}
        >
          Clear App Data
        </p>
        <div className="flex gap-2">
          <Input
            type="text"
            value={clearPackage}
            onChange={(_e, data) => setClearPackage(data.value)}
            placeholder="Package name"
            className="flex-1 font-mono text-xs"
            size="small"
          />
          <Button
            appearance="outline"
            size="small"
            icon={<BroomRegular />}
            onClick={handleClearData}
            disabled={isAnyPending || !clearPackage.trim()}
          >
            Clear
          </Button>
        </div>
      </div>

      {statusMessage && (
        <p
          className="text-xs text-center mt-3 py-1 rounded"
          style={{
            color: tokens.colorStatusSuccessForeground1,
            backgroundColor: tokens.colorStatusSuccessBackground1,
          }}
        >
          {statusMessage}
        </p>
      )}
    </Card>
  );
};
