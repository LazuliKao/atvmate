import axios from 'axios';

export interface ApiResponse {
  success: boolean;
  message: string;
}

export interface DiscoveredDevice {
  id: string;
  identifier: string;
  connection_type: string;
  state: string;
}

export interface DeviceInfoData {
  model: string;
  android_version: string;
  serial: string;
  screen_resolution: string;
}

export interface OperationRecord {
  timestamp: string;
  operation: string;
  params: string;
  success: boolean;
}

export interface DeviceStatus {
  device_id: string;
  device_type: string;
  connected: boolean;
}

export interface BatchResult {
  device_id: string;
  success: boolean;
  message: string;
}

const client = axios.create({ baseURL: '/api' });

function pathPart(value: string): string {
  return encodeURIComponent(value);
}

function requireSuccess(response: ApiResponse): ApiResponse {
  if (!response.success) {
    throw new Error(response.message);
  }
  return response;
}

export const api = {
  async listDevices(): Promise<string[]> {
    return (await client.get<{ devices: string[] }>('/devices')).data.devices;
  },
  async discoverDevices(): Promise<DiscoveredDevice[]> {
    return (await client.get<{ devices: DiscoveredDevice[] }>('/devices/discover')).data.devices;
  },
  async addDevice(ip: string, port: number): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>('/devices', { ip, port })).data);
  },
  async addUsbDevice(): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>('/devices/usb', {})).data);
  },
  async addServerDevice(serial: string, serverAddr?: string): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>('/devices/server', {
      serial,
      server_addr: serverAddr || undefined,
    })).data);
  },
  async removeDevice(deviceId: string): Promise<ApiResponse> {
    return requireSuccess((await client.delete<ApiResponse>(`/devices/${pathPart(deviceId)}`)).data);
  },
  async sendKey(deviceId: string, key: string): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/key/${pathPart(key)}`)).data);
  },
  async getDeviceInfo(deviceId: string): Promise<DeviceInfoData> {
    const response = (await client.get<{ success: boolean; message: string; device_info: DeviceInfoData | null }>(`/devices/${pathPart(deviceId)}/info`)).data;
    if (!response.success || !response.device_info) {
      throw new Error(response.message);
    }
    return response.device_info;
  },
  async listApps(deviceId: string): Promise<string[]> {
    const response = (await client.get<{ success: boolean; message: string; apps: string[] }>(`/devices/${pathPart(deviceId)}/apps`)).data;
    if (!response.success) {
      throw new Error(response.message);
    }
    return response.apps;
  },
  async launchApp(deviceId: string, packageName: string): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/apps/${pathPart(packageName)}/launch`)).data);
  },
  async uninstallApp(deviceId: string, packageName: string): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/apps/${pathPart(packageName)}/uninstall`)).data);
  },
  async installApp(deviceId: string, file: File): Promise<ApiResponse> {
    const form = new FormData();
    form.append('file', file);
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/apps/install`, form)).data);
  },
  async getHistory(deviceId: string): Promise<OperationRecord[]> {
    return (await client.get<{ operations: OperationRecord[] }>(`/devices/${pathPart(deviceId)}/history`)).data.operations;
  },
  async replay(deviceId: string, operations: OperationRecord[]): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/history/replay`, { operations })).data);
  },
  async batchKey(devices: string[], key: string): Promise<BatchResult[]> {
    return (await client.post<{ results: BatchResult[] }>('/devices/batch/key', { devices, key })).data.results;
  },
  async batchCommand(devices: string[], command: string): Promise<BatchResult[]> {
    return (await client.post<{ results: BatchResult[] }>('/devices/batch/command', { devices, command })).data.results;
  },
  async getStatus(): Promise<DeviceStatus[]> {
    return (await client.get<DeviceStatus[]>('/devices/status')).data;
  },
  async reboot(deviceId: string, mode = ''): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/reboot${mode}`)).data);
  },
  async clearAppData(deviceId: string, packageName: string): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/apps/${pathPart(packageName)}/clear`)).data);
  },
  async pushFile(deviceId: string, file: File, remotePath: string): Promise<ApiResponse> {
    const form = new FormData();
    form.append('file', file);
    form.append('remote_path', remotePath);
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/push`, form)).data);
  },
  async pullFile(deviceId: string, remotePath: string): Promise<Blob> {
    const response = await client.get<Blob>(`/devices/${pathPart(deviceId)}/pull`, {
      params: { path: remotePath },
      responseType: 'blob',
    });
    return binaryResponse(response.data, response.headers['content-type']);
  },
  async takeScreenshot(deviceId: string): Promise<Blob> {
    const response = await client.get<Blob>(`/devices/${pathPart(deviceId)}/screenshot`, {
      responseType: 'blob',
    });
    return binaryResponse(response.data, response.headers['content-type']);
  },
  async sendText(deviceId: string, text: string): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/input/text`, { text })).data);
  },
  async scroll(deviceId: string, direction: 'up' | 'down', amount: number): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/input/mouse`, { direction, amount })).data);
  },
  async gesture(deviceId: string, points: Array<{ x: number; y: number }>, durationMs: number): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/input/gesture`, { points, duration_ms: durationMs })).data);
  },
  async sendMappedKey(deviceId: string, key: string, keycode: number): Promise<ApiResponse> {
    return requireSuccess((await client.post<ApiResponse>(`/devices/${pathPart(deviceId)}/input/keymap`, { key, mapping: { [key]: keycode } })).data);
  },
};

async function binaryResponse(data: Blob, contentType?: string): Promise<Blob> {
  if (contentType?.includes('application/json')) {
    const response = JSON.parse(await data.text()) as ApiResponse;
    throw new Error(response.message);
  }
  return data;
}
