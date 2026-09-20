import type { FunctionalComponent } from 'preact';
import { useRef, useState } from 'preact/hooks';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Input, tokens, Spinner } from '@fluentui/react-components';
import {
  ArrowClockwiseRegular,
  AppsList24Regular,
  PlayRegular,
  DeleteRegular,
  SearchRegular,
} from '@fluentui/react-icons';
import { api } from '../api';

interface AppManagerProps {
  deviceId: string;
}

export const AppManager: FunctionalComponent<AppManagerProps> = ({ deviceId }) => {
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState('');
  const apkInputRef = useRef<HTMLInputElement>(null);

  const { data: apps, isLoading, isError, refetch } = useQuery({
    queryKey: ['apps', deviceId],
    queryFn: async () => {
      return api.listApps(deviceId);
    },
    enabled: !!deviceId,
  });

  const launchMutation = useMutation({
    mutationFn: (packageName: string) => api.launchApp(deviceId, packageName),
  });

  const uninstallMutation = useMutation({
    mutationFn: (packageName: string) => api.uninstallApp(deviceId, packageName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['apps', deviceId] });
    },
  });
  const installMutation = useMutation({
    mutationFn: (file: File) => api.installApp(deviceId, file),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['apps', deviceId] }),
  });

  const filteredApps = (apps || []).filter(
    (packageName) => packageName.toLowerCase().includes(filter.toLowerCase())
  );

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
          <AppsList24Regular style={{ color: tokens.colorBrandForeground1 }} />
          <h3
            className="text-sm font-semibold"
            style={{ color: tokens.colorBrandForeground1 }}
          >
            Apps
          </h3>
        </div>
        <Button
          appearance="subtle"
          size="small"
          icon={<ArrowClockwiseRegular />}
          onClick={() => refetch()}
          aria-label="Refresh apps"
        />
      </div>

      <input
        ref={apkInputRef}
        type="file"
        accept=".apk,application/vnd.android.package-archive"
        className="hidden"
        onChange={(event) => {
          const file = (event.target as HTMLInputElement).files?.[0];
          if (file) installMutation.mutate(file);
          event.currentTarget.value = '';
        }}
      />
      <Button
        appearance="outline"
        size="small"
        className="w-full mb-3"
        onClick={() => apkInputRef.current?.click()}
        disabled={installMutation.isPending}
      >
        {installMutation.isPending ? 'Installing APK...' : 'Install APK'}
      </Button>

      <div className="mb-3">
        <Input
          type="text"
          value={filter}
          onChange={(_e, data) => setFilter(data.value)}
          placeholder="Filter apps..."
          contentBefore={<SearchRegular />}
          className="w-full font-mono text-xs"
          size="small"
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
          Failed to load apps
        </p>
      )}

      {apps && (
        <div className="space-y-1 max-h-64 overflow-y-auto">
          {filteredApps.length === 0 ? (
            <p
              className="text-xs text-center py-3"
              style={{ color: tokens.colorNeutralForeground2 }}
            >
              {filter ? 'No matching apps' : 'No apps found'}
            </p>
          ) : (
            filteredApps.map((packageName) => (
              <div
                key={packageName}
                className="flex items-center justify-between p-2 rounded-lg"
                style={{ backgroundColor: tokens.colorNeutralBackground1 }}
              >
                <div className="flex-1 min-w-0 mr-2">
                  <p className="text-xs font-medium truncate">{packageName}</p>
                  <p
                    className="text-[10px] font-mono truncate"
                    style={{ color: tokens.colorNeutralForeground2 }}
                  >
                    Package from `pm list packages`
                  </p>
                </div>
                <div className="flex gap-1">
                  <Button
                    appearance="subtle"
                    size="small"
                    icon={<PlayRegular />}
                    onClick={() => launchMutation.mutate(packageName)}
                    disabled={launchMutation.isPending}
                    title="Launch"
                    aria-label={`Launch ${packageName}`}
                  />
                  <Button
                    appearance="subtle"
                    size="small"
                    icon={<DeleteRegular />}
                    onClick={() => {
                      if (confirm(`Uninstall ${packageName}?`)) uninstallMutation.mutate(packageName);
                    }}
                    disabled={uninstallMutation.isPending}
                    title="Uninstall"
                    aria-label={`Uninstall ${packageName}`}
                    style={{ color: tokens.colorStatusDangerForeground1 }}
                  />
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {(launchMutation.isError || uninstallMutation.isError || installMutation.isError) && (
        <p
          className="text-xs text-center mt-2"
          style={{ color: tokens.colorStatusDangerForeground1 }}
        >
          Operation failed
        </p>
      )}
    </Card>
  );
};
