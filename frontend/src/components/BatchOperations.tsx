import type { FunctionalComponent } from 'preact';
import { useState, useCallback } from 'preact/hooks';
import { useMutation } from '@tanstack/react-query';
import { Button, Card, Input, tokens, Spinner, Badge } from '@fluentui/react-components';
import {
  People24Regular,
  SendRegular,
  WindowConsoleRegular,
} from '@fluentui/react-icons';
import { api } from '../api';

interface BatchOperationsProps {
  deviceId: string;
  devices: string[];
}

export const BatchOperations: FunctionalComponent<BatchOperationsProps> = ({
  deviceId,
  devices,
}) => {
  const [keyName, setKeyName] = useState('');
  const [command, setCommand] = useState('');
  const [selectedDevices, setSelectedDevices] = useState<string[]>([deviceId]);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const batchKeyMutation = useMutation({
    mutationFn: () => api.batchKey(selectedDevices, keyName.trim()),
    onSuccess: (results) => {
      const failed = results.filter((result) => !result.success);
      setStatusMessage(failed.length ? `${failed.length} device(s) failed: ${failed[0].message}` : `Sent key "${keyName}" to ${selectedDevices.length} device(s)`);
      setKeyName('');
    },
    onError: () => setStatusMessage('Batch key press failed'),
  });

  const batchCommandMutation = useMutation({
    mutationFn: () => api.batchCommand(selectedDevices, command.trim()),
    onSuccess: (results) => {
      const failed = results.filter((result) => !result.success);
      setStatusMessage(failed.length ? `${failed.length} device(s) failed: ${failed[0].message}` : `Sent command to ${selectedDevices.length} device(s)`);
      setCommand('');
    },
    onError: () => setStatusMessage('Batch command failed'),
  });

  const toggleDevice = useCallback(
    (id: string) => {
      setSelectedDevices((prev) =>
        prev.includes(id) ? prev.filter((d) => d !== id) : [...prev, id]
      );
    },
    []
  );

  const selectAll = useCallback(() => {
    setSelectedDevices([...devices]);
  }, [devices]);

  const selectNone = useCallback(() => {
    setSelectedDevices([]);
  }, []);

  const isAnyPending = batchKeyMutation.isPending || batchCommandMutation.isPending;

  return (
    <Card
      className="w-full"
      style={{
        backgroundColor: tokens.colorNeutralBackground2,
        color: tokens.colorNeutralForeground1,
      }}
    >
      <div className="flex items-center gap-2 mb-3">
        <People24Regular style={{ color: tokens.colorBrandForeground1 }} />
        <h3
          className="text-sm font-semibold"
          style={{ color: tokens.colorBrandForeground1 }}
        >
          Batch Operations
        </h3>
      </div>

      {/* Device Selection */}
      <div className="mb-3">
        <div className="flex items-center justify-between mb-2">
          <p
            className="text-xs font-medium"
            style={{ color: tokens.colorNeutralForeground2 }}
          >
            Target Devices ({selectedDevices.length}/{devices.length})
          </p>
          <div className="flex gap-1">
            <Button
              appearance="subtle"
              size="small"
              onClick={selectAll}
            >
              All
            </Button>
            <Button
              appearance="subtle"
              size="small"
              onClick={selectNone}
            >
              None
            </Button>
          </div>
        </div>
        <div className="flex flex-wrap gap-1">
          {devices.map((id) => (
            <Badge
              key={id}
              appearance={selectedDevices.includes(id) ? 'filled' : 'outline'}
              size="small"
              color={selectedDevices.includes(id) ? 'brand' : 'subtle'}
              onClick={() => toggleDevice(id)}
              className="cursor-pointer font-mono text-[10px]"
            >
              {id}
            </Badge>
          ))}
        </div>
      </div>

      {/* Batch Key */}
      <div className="mb-3">
        <p
          className="text-xs font-medium mb-2"
          style={{ color: tokens.colorNeutralForeground2 }}
        >
          Send Key to All Selected
        </p>
        <div className="flex gap-2">
          <Input
            type="text"
            value={keyName}
            onChange={(_e, data) => setKeyName(data.value)}
            placeholder="Key name (e.g., home, back)"
            className="flex-1 font-mono text-xs"
            size="small"
          />
          <Button
            appearance="primary"
            size="small"
            icon={batchKeyMutation.isPending ? <Spinner size="tiny" /> : <SendRegular />}
            onClick={() => batchKeyMutation.mutate()}
            disabled={isAnyPending || !keyName.trim() || selectedDevices.length === 0}
          >
            Send
          </Button>
        </div>
      </div>

      <div
        className="border-t pt-3"
        style={{ borderColor: tokens.colorNeutralStroke2 }}
      >
        <p
          className="text-xs font-medium mb-2"
          style={{ color: tokens.colorNeutralForeground2 }}
        >
          Run Command on All Selected
        </p>
        <div className="flex gap-2">
          <Input
            type="text"
            value={command}
            onChange={(_e, data) => setCommand(data.value)}
            placeholder="Shell command"
            className="flex-1 font-mono text-xs"
            size="small"
            contentBefore={<WindowConsoleRegular />}
          />
          <Button
            appearance="primary"
            size="small"
            icon={batchCommandMutation.isPending ? <Spinner size="tiny" /> : <SendRegular />}
            onClick={() => batchCommandMutation.mutate()}
            disabled={isAnyPending || !command.trim() || selectedDevices.length === 0}
          >
            Run
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
