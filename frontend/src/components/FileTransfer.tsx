import type { FunctionalComponent } from 'preact';
import { useState, useCallback, useRef } from 'preact/hooks';
import { useMutation } from '@tanstack/react-query';
import { Button, Card, Input, tokens, Spinner } from '@fluentui/react-components';
import {
  ArrowUpload24Regular,
  ArrowDownload24Regular,
  Folder24Regular,
} from '@fluentui/react-icons';
import { api } from '../api';

interface FileTransferProps {
  deviceId: string;
}

export const FileTransfer: FunctionalComponent<FileTransferProps> = ({ deviceId }) => {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [remotePath, setRemotePath] = useState('');
  const [downloadPath, setDownloadPath] = useState('');
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);

  const uploadMutation = useMutation({
    mutationFn: async () => {
      if (!selectedFile || !remotePath.trim()) {
        throw new Error('File and remote path required');
      }

      return api.pushFile(deviceId, selectedFile, remotePath.trim());
    },
    onSuccess: () => {
      setStatusMessage(`Uploaded to ${remotePath}`);
      setSelectedFile(null);
      setRemotePath('');
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    },
    onError: () => setStatusMessage('Upload failed'),
  });

  const downloadMutation = useMutation({
    mutationFn: async () => {
      if (!downloadPath.trim()) {
        throw new Error('Remote path required');
      }

      return api.pullFile(deviceId, downloadPath.trim());
    },
    onSuccess: (blob) => {
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = downloadPath.split('/').pop() || 'download';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      setStatusMessage(`Downloaded ${downloadPath}`);
      setDownloadPath('');
    },
    onError: () => setStatusMessage('Download failed'),
  });

  const handleFileSelect = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback((e: Event) => {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file) {
      setSelectedFile(file);
      if (!remotePath) {
        setRemotePath(`/sdcard/${file.name}`);
      }
    }
  }, [remotePath]);

  return (
    <Card
      className="w-full"
      style={{
        backgroundColor: tokens.colorNeutralBackground2,
        color: tokens.colorNeutralForeground1,
      }}
    >
      <div className="flex items-center gap-2 mb-3">
        <Folder24Regular style={{ color: tokens.colorBrandForeground1 }} />
        <h3
          className="text-sm font-semibold"
          style={{ color: tokens.colorBrandForeground1 }}
        >
          File Transfer
        </h3>
      </div>

      {/* Upload Section */}
      <div className="mb-4">
        <p
          className="text-xs font-medium mb-2"
          style={{ color: tokens.colorNeutralForeground2 }}
        >
          Upload to Device
        </p>
        <input
          ref={fileInputRef}
          type="file"
          onChange={handleFileChange}
          className="hidden"
        />
        <div className="space-y-2">
          <Button
            appearance="outline"
            size="small"
            icon={<ArrowUpload24Regular />}
            onClick={handleFileSelect}
            className="w-full"
          >
            {selectedFile ? selectedFile.name : 'Choose File'}
          </Button>
          <Input
            type="text"
            value={remotePath}
            onChange={(_e, data) => setRemotePath(data.value)}
            placeholder="Remote path (e.g., /sdcard/file.txt)"
            className="w-full font-mono text-xs"
            size="small"
          />
          <Button
            appearance="primary"
            size="small"
            icon={uploadMutation.isPending ? <Spinner size="tiny" /> : <ArrowUpload24Regular />}
            onClick={() => uploadMutation.mutate()}
            disabled={uploadMutation.isPending || !selectedFile || !remotePath.trim()}
            className="w-full"
          >
            {uploadMutation.isPending ? 'Uploading...' : 'Upload'}
          </Button>
        </div>
      </div>

      <div
        className="border-t pt-4"
        style={{ borderColor: tokens.colorNeutralStroke2 }}
      >
        <p
          className="text-xs font-medium mb-2"
          style={{ color: tokens.colorNeutralForeground2 }}
        >
          Download from Device
        </p>
        <div className="space-y-2">
          <Input
            type="text"
            value={downloadPath}
            onChange={(_e, data) => setDownloadPath(data.value)}
            placeholder="Remote path (e.g., /sdcard/file.txt)"
            className="w-full font-mono text-xs"
            size="small"
          />
          <Button
            appearance="primary"
            size="small"
            icon={downloadMutation.isPending ? <Spinner size="tiny" /> : <ArrowDownload24Regular />}
            onClick={() => downloadMutation.mutate()}
            disabled={downloadMutation.isPending || !downloadPath.trim()}
            className="w-full"
          >
            {downloadMutation.isPending ? 'Downloading...' : 'Download'}
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
