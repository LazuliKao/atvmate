import { Button, Input } from '@fluentui/react-components';
import { AddRegular, DeleteRegular, ArrowClockwiseRegular } from '@fluentui/react-icons';
import type { FunctionalComponent } from 'preact';
import { useState } from 'preact/hooks';
import {
  useGetDevices,
  usePostDevices,
  useDeleteDevicesIp
} from '../api/default/default';
import type { DeviceList } from '../api/model';

interface DeviceManagerProps {
  onSelectDevice: (ip: string) => void;
}

export const DeviceManager: FunctionalComponent<DeviceManagerProps> = ({ onSelectDevice }) => {
  const [ip, setIp] = useState('');
  const [port, setPort] = useState('5555');

  const {
    data: devicesResponse,
    isLoading,
    isError,
    refetch
  } = useGetDevices();

  const addDeviceMutation = usePostDevices({
    mutation: {
      onSuccess: () => {
        setIp('');
        setPort('5555');
        refetch();
      },
    },
  });

  const removeDeviceMutation = useDeleteDevicesIp({
    mutation: {
      onSuccess: () => {
        refetch();
      },
    },
  });

  const handleAddDevice = (e: Event) => {
    e.preventDefault();
    if (ip && port) {
      addDeviceMutation.mutate({
        data: {
          ip,
          port: parseInt(port, 10),
        },
      });
    }
  };

  const handleRemoveDevice = (deviceIp: string, e: Event) => {
    e.stopPropagation();
    if (confirm(`Remove device ${deviceIp}?`)) {
      removeDeviceMutation.mutate({ ip: deviceIp });
    }
  };

  const devices = (devicesResponse?.data as unknown as DeviceList)?.devices || [];

  return (
    <div className="p-4 bg-gray-900 text-white rounded-xl w-full max-w-md mx-auto shadow-xl border border-gray-800">
      <h2 className="text-xl font-bold mb-6 text-center text-blue-400 tracking-tight">Device Manager</h2>

      <form onSubmit={handleAddDevice} className="mb-8 space-y-4 bg-gray-800/50 p-4 rounded-lg border border-gray-700/50">
        <div>
          <label className="block text-xs font-bold text-gray-400 mb-1.5 uppercase tracking-wider">IP Address</label>
          <Input
            type="text"
            value={ip}
            onChange={(_e, data) => setIp(data.value)}
            placeholder="192.168.1.10"
            className="w-full p-3 rounded-lg bg-gray-900 text-white border border-gray-700 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none min-h-[44px] transition-all placeholder-gray-600 font-mono text-sm"
            required
          />
        </div>
        <div>
          <label className="block text-xs font-bold text-gray-400 mb-1.5 uppercase tracking-wider">Port</label>
          <Input
            type="number"
            value={port}
            onChange={(_e, data) => setPort(data.value)}
            placeholder="5555"
            className="w-full p-3 rounded-lg bg-gray-900 text-white border border-gray-700 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none min-h-[44px] transition-all placeholder-gray-600 font-mono text-sm"
            required
          />
        </div>
        <Button
          type="submit"
          disabled={addDeviceMutation.isPending}
          className={`w-full p-3 rounded-lg font-bold text-white min-h-[44px] flex items-center justify-center transition-all mt-2 ${
            addDeviceMutation.isPending
              ? 'bg-blue-900/50 cursor-not-allowed text-blue-300'
              : 'bg-blue-600 hover:bg-blue-500 active:bg-blue-700 shadow-lg hover:shadow-blue-500/25'
          }`}
          icon={!addDeviceMutation.isPending ? <AddRegular /> : undefined}
        >
          {addDeviceMutation.isPending ? (
            <span className="flex items-center gap-2">
              <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
              Adding...
            </span>
          ) : 'Add Device'}
        </Button>
        {addDeviceMutation.isError && (
          <p className="text-red-400 text-xs text-center mt-2 bg-red-900/10 py-1 rounded">
            Failed to add device. Check connection.
          </p>
        )}
      </form>

      <div className="space-y-3">
        <h3 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-2 px-1">Saved Devices</h3>

        {isLoading && (
          <div className="flex flex-col items-center justify-center py-8 text-gray-500 space-y-2">
             <div className="w-6 h-6 border-2 border-blue-500/30 border-t-blue-500 rounded-full animate-spin"></div>
             <span className="text-xs">Loading devices...</span>
          </div>
        )}

        {isError && (
          <div className="bg-red-900/20 border border-red-800/50 text-red-300 p-4 rounded-lg text-center">
            <p className="mb-3 text-sm">Unable to load devices</p>
            <Button
              onClick={() => refetch()}
              className="px-4 py-2 bg-red-800 hover:bg-red-700 rounded-lg text-xs font-bold uppercase tracking-wide transition-colors"
              icon={<ArrowClockwiseRegular />}
            >
              Retry Connection
            </Button>
          </div>
        )}

        {!isLoading && !isError && (
          <div className="space-y-2">
            {devices.length === 0 ? (
              <div className="text-center py-8 bg-gray-800/30 rounded-lg border border-gray-800 border-dashed">
                <p className="text-gray-500 text-sm">No devices found.</p>
                <p className="text-gray-600 text-xs mt-1">Add a device above to get started.</p>
              </div>
            ) : (
              devices.map((deviceIp) => (
                <div
                  key={deviceIp}
                  className="group flex items-center justify-between p-3 bg-gray-800 hover:bg-gray-750 border border-gray-700 hover:border-gray-600 rounded-lg transition-all"
                >
                  <Button
                    onClick={() => onSelectDevice(deviceIp)}
                    appearance="transparent"
                    className="flex-1 text-left font-mono text-blue-300 hover:text-blue-100 truncate mr-3 min-h-[44px] flex items-center outline-none group-hover:translate-x-1 transition-transform"
                  >
                    <span className="w-2 h-2 rounded-full bg-green-500 mr-3 shadow-[0_0_8px_rgba(34,197,94,0.4)]"></span>
                    {deviceIp}
                  </Button>
                  <Button
                    onClick={(e) => handleRemoveDevice(deviceIp, e)}
                    disabled={removeDeviceMutation.isPending}
                    className="w-10 h-10 flex items-center justify-center text-gray-500 hover:text-red-400 hover:bg-red-900/20 rounded-lg transition-colors outline-none focus:ring-1 focus:ring-red-500/50"
                    aria-label={`Remove ${deviceIp}`}
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
