import { Button, Input, Tab, TabList, tokens } from '@fluentui/react-components';
import {
  AddRegular,
  DeleteRegular,
  ArrowClockwiseRegular,
  SearchRegular,
  PlugConnectedRegular,
} from '@fluentui/react-icons';
import type { FunctionalComponent } from 'preact';
import { useState } from 'preact/hooks';
import {
  useGetDevices,
  useDiscoverDevices,
  usePostDevices,
  usePostDevicesUsb,
  usePostDevicesServer,
  useDeleteDevicesDeviceId,
} from '../api/default/default';
import type { DeviceList, DeviceDiscovery, DiscoveredDeviceResponse } from '../api/model';

type ConnectionType = 'tcp' | 'usb' | 'server';

interface DeviceManagerProps {
  onSelectDevice: (deviceId: string) => void;
}

export const DeviceManager: FunctionalComponent<DeviceManagerProps> = ({ onSelectDevice }) => {
  const [connectionType, setConnectionType] = useState<ConnectionType>('tcp');

  // TCP state
  const [ip, setIp] = useState('');
  const [port, setPort] = useState('5555');

  // Server state
  const [serial, setSerial] = useState('');
  const [serverAddr, setServerAddr] = useState('');

  const {
    data: devicesResponse,
    isLoading,
    isError,
    refetch,
  } = useGetDevices();

  const {
    data: discoveredResponse,
    isLoading: isDiscovering,
    refetch: refetchDiscovery,
  } = useDiscoverDevices({
    query: {
      enabled: false, // Only fetch when manually triggered
    },
  });

  const addTcpDeviceMutation = usePostDevices({
    mutation: {
      onSuccess: () => {
        setIp('');
        setPort('5555');
        refetch();
      },
    },
  });

  const addUsbDeviceMutation = usePostDevicesUsb({
    mutation: {
      onSuccess: () => {
        refetch();
      },
    },
  });

  const addServerDeviceMutation = usePostDevicesServer({
    mutation: {
      onSuccess: () => {
        setSerial('');
        setServerAddr('');
        refetch();
      },
    },
  });

  const removeDeviceMutation = useDeleteDevicesDeviceId({
    mutation: {
      onSuccess: () => {
        refetch();
      },
    },
  });

  const handleAddTcpDevice = (e: Event) => {
    e.preventDefault();
    if (ip && port) {
      addTcpDeviceMutation.mutate({
        data: { ip, port: parseInt(port, 10) },
      });
    }
  };

  const handleAddUsbDevice = () => {
    addUsbDeviceMutation.mutate({
      data: {}, // Auto-detect
    });
  };

  const handleAddServerDevice = (e: Event) => {
    e.preventDefault();
    if (serial) {
      addServerDeviceMutation.mutate({
        data: {
          serial,
          server_addr: serverAddr || undefined,
        },
      });
    }
  };

  const handleConnectDiscovered = (device: DiscoveredDeviceResponse) => {
    // For discovered devices, we just select them directly
    onSelectDevice(device.id);
  };

  const handleRemoveDevice = (deviceId: string, e: any) => {
    e.stopPropagation();
    if (confirm(`Remove device ${deviceId}?`)) {
      removeDeviceMutation.mutate({ deviceId });
    }
  };

  const devices = (devicesResponse?.data as unknown as DeviceList)?.devices || [];
  const discovered = (discoveredResponse?.data as unknown as DeviceDiscovery)?.devices || [];

  const isAnyMutationPending =
    addTcpDeviceMutation.isPending ||
    addUsbDeviceMutation.isPending ||
    addServerDeviceMutation.isPending;

  return (
    <div
      className="w-full"
      style={{
        backgroundColor: tokens.colorNeutralBackground2,
        color: tokens.colorNeutralForeground1,
      }}
    >
      <h2
        className="text-lg font-semibold mb-4 text-center"
        style={{ color: tokens.colorBrandForeground1 }}
      >
        Devices
      </h2>

      {/* Connection Type Tabs */}
      <TabList
        selectedValue={connectionType}
        onTabSelect={(_, data) => setConnectionType(data.value as ConnectionType)}
        size="small"
        className="mb-4"
      >
        <Tab value="tcp">TCP/IP</Tab>
        <Tab value="usb">USB</Tab>
        <Tab value="server">ADB Server</Tab>
      </TabList>

      {/* TCP Connection Form */}
      {connectionType === 'tcp' && (
        <form onSubmit={handleAddTcpDevice} className="mb-4 space-y-3">
          <div className="flex gap-2">
            <Input
              type="text"
              value={ip}
              onChange={(_e, data) => setIp(data.value)}
              placeholder="IP Address"
              className="flex-1 font-mono text-sm"
              required
            />
            <Input
              type="number"
              value={port}
              onChange={(_e, data) => setPort(data.value)}
              placeholder="Port"
              className="w-24 font-mono text-sm"
              required
            />
          </div>
          <Button
            type="submit"
            disabled={isAnyMutationPending}
            appearance="primary"
            className="w-full"
            size="small"
            icon={!isAnyMutationPending ? <AddRegular /> : undefined}
          >
            {addTcpDeviceMutation.isPending ? 'Connecting...' : 'Add TCP Device'}
          </Button>
        </form>
      )}

      {/* USB Connection */}
      {connectionType === 'usb' && (
        <div className="mb-4 space-y-3">
          <Button
            onClick={handleAddUsbDevice}
            disabled={isAnyMutationPending}
            appearance="primary"
            className="w-full"
            size="small"
            icon={<PlugConnectedRegular />}
          >
            {addUsbDeviceMutation.isPending ? 'Connecting...' : 'Connect USB Device'}
          </Button>
          <p className="text-xs text-center" style={{ color: tokens.colorNeutralForeground2 }}>
            Auto-detects connected USB devices
          </p>
        </div>
      )}

      {/* ADB Server Connection Form */}
      {connectionType === 'server' && (
        <form onSubmit={handleAddServerDevice} className="mb-4 space-y-3">
          <Input
            type="text"
            value={serial}
            onChange={(_e, data) => setSerial(data.value)}
            placeholder="Device Serial (e.g., emulator-5554)"
            className="font-mono text-sm"
            required
          />
          <Input
            type="text"
            value={serverAddr}
            onChange={(_e, data) => setServerAddr(data.value)}
            placeholder="Server Address (optional, default: 127.0.0.1:5037)"
            className="font-mono text-sm"
          />
          <Button
            type="submit"
            disabled={isAnyMutationPending}
            appearance="primary"
            className="w-full"
            size="small"
            icon={!isAnyMutationPending ? <AddRegular /> : undefined}
          >
            {addServerDeviceMutation.isPending ? 'Connecting...' : 'Add Server Device'}
          </Button>
        </form>
      )}

      {/* Error Messages */}
      {(addTcpDeviceMutation.isError ||
        addUsbDeviceMutation.isError ||
        addServerDeviceMutation.isError) && (
        <p
          className="text-xs text-center mb-3"
          style={{ color: tokens.colorStatusDangerForeground1 }}
        >
          Failed to connect device
        </p>
      )}

      {/* Device Discovery Section */}
      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <h3
            className="text-xs font-medium"
            style={{ color: tokens.colorNeutralForeground2 }}
          >
            Discovered
          </h3>
          <Button
            onClick={() => refetchDiscovery()}
            disabled={isDiscovering}
            appearance="subtle"
            size="small"
            icon={<SearchRegular />}
          >
            {isDiscovering ? 'Scanning...' : 'Scan'}
          </Button>
        </div>

        {discovered.length > 0 && (
          <div className="space-y-1 mb-3">
            {discovered.map((device) => (
              <div
                key={device.id}
                className="flex items-center justify-between p-2 rounded-lg cursor-pointer hover:opacity-80"
                style={{ backgroundColor: tokens.colorNeutralBackground1 }}
                onClick={() => handleConnectDiscovered(device)}
              >
                <div className="flex-1 min-w-0">
                  <p
                    className="text-xs font-mono truncate"
                    style={{ color: tokens.colorBrandForeground1 }}
                  >
                    {device.identifier}
                  </p>
                  <p
                    className="text-xs"
                    style={{ color: tokens.colorNeutralForeground2 }}
                  >
                    {device.connection_type} • {device.state}
                  </p>
                </div>
                <PlugConnectedRegular
                  style={{ color: tokens.colorNeutralForeground2 }}
                />
              </div>
            ))}
          </div>
        )}

        {discovered.length === 0 && !isDiscovering && (
          <p
            className="text-xs text-center py-2"
            style={{ color: tokens.colorNeutralForeground2 }}
          >
            Click Scan to discover devices
          </p>
        )}
      </div>

      {/* Connected Devices List */}
      <div>
        <h3
          className="text-xs font-medium mb-2 px-1"
          style={{ color: tokens.colorNeutralForeground2 }}
        >
          Connected
        </h3>

        {isLoading && (
          <div className="flex items-center justify-center py-4 gap-2">
            <div className="w-4 h-4 border-2 border-blue-500/30 border-t-blue-500 rounded-full animate-spin"></div>
            <span className="text-xs">Loading...</span>
          </div>
        )}

        {isError && (
          <div
            className="p-3 text-center rounded-lg"
            style={{ backgroundColor: tokens.colorStatusDangerBackground1 }}
          >
            <p
              className="text-xs mb-2"
              style={{ color: tokens.colorStatusDangerForeground1 }}
            >
              Connection failed
            </p>
            <Button
              onClick={() => refetch()}
              appearance="subtle"
              size="small"
              icon={<ArrowClockwiseRegular />}
            >
              Retry
            </Button>
          </div>
        )}

        {!isLoading && !isError && (
          <div className="space-y-2">
            {devices.length === 0 ? (
              <div
                className="text-center py-4 rounded-lg"
                style={{ backgroundColor: tokens.colorNeutralBackground1 }}
              >
                <p
                  className="text-xs"
                  style={{ color: tokens.colorNeutralForeground2 }}
                >
                  No devices connected
                </p>
              </div>
            ) : (
              devices.map((deviceId) => (
                <div
                  key={deviceId}
                  className="group flex items-center justify-between p-2 rounded-lg"
                  style={{ backgroundColor: tokens.colorNeutralBackground1 }}
                >
                  <Button
                    onClick={() => onSelectDevice(deviceId)}
                    appearance="transparent"
                    className="flex-1 text-left font-mono truncate"
                    style={{ color: tokens.colorBrandForeground1 }}
                  >
                    <span className="w-2 h-2 rounded-full bg-green-500 mr-2"></span>
                    {deviceId}
                  </Button>
                  <Button
                    onClick={(e) => handleRemoveDevice(deviceId, e)}
                    disabled={removeDeviceMutation.isPending}
                    appearance="transparent"
                    size="small"
                    style={{ color: tokens.colorNeutralForeground2 }}
                    aria-label={`Remove ${deviceId}`}
                    icon={<DeleteRegular />}
                  />
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
};
