import type { FunctionalComponent } from 'preact';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Button, Card, tokens, Spinner, Badge } from '@fluentui/react-components';
import {
  ArrowClockwiseRegular,
  History24Regular,
  PlayRegular,
} from '@fluentui/react-icons';
import { api, type OperationRecord } from '../api';

interface HistoryViewerProps {
  deviceId: string;
}

export const HistoryViewer: FunctionalComponent<HistoryViewerProps> = ({ deviceId }) => {
  const queryClient = useQueryClient();

  const { data: history, isLoading, isError, refetch } = useQuery({
    queryKey: ['history', deviceId],
    queryFn: async () => {
      return api.getHistory(deviceId);
    },
    enabled: !!deviceId,
  });

  const replayMutation = useMutation({
    mutationFn: (record: OperationRecord) => api.replay(deviceId, [record]),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['history', deviceId] });
    },
  });

  const formatTimestamp = (ts: string) => {
    try {
      const date = new Date(ts);
      return date.toLocaleString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      });
    } catch {
      return ts;
    }
  };

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
          <History24Regular style={{ color: tokens.colorBrandForeground1 }} />
          <h3
            className="text-sm font-semibold"
            style={{ color: tokens.colorBrandForeground1 }}
          >
            Operation History
          </h3>
        </div>
        <Button
          appearance="subtle"
          size="small"
          icon={<ArrowClockwiseRegular />}
          onClick={() => refetch()}
          aria-label="Refresh history"
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
          Failed to load history
        </p>
      )}

      {history && (
        <div className="space-y-1 max-h-64 overflow-y-auto">
          {history.length === 0 ? (
            <p
              className="text-xs text-center py-3"
              style={{ color: tokens.colorNeutralForeground2 }}
            >
              No operations recorded
            </p>
          ) : (
            history.map((record, index) => (
              <div
                key={`${record.timestamp}-${index}`}
                className="flex items-center justify-between p-2 rounded-lg"
                style={{ backgroundColor: tokens.colorNeutralBackground1 }}
              >
                <div className="flex-1 min-w-0 mr-2">
                  <div className="flex items-center gap-2">
                    <p className="text-xs font-medium truncate">{record.operation}</p>
                    <Badge
                      appearance="outline"
                      size="small"
                      color="brand"
                    >
                      {formatTimestamp(record.timestamp)}
                    </Badge>
                  </div>
                  <p
                    className="text-[10px] truncate mt-0.5"
                    style={{ color: record.success ? tokens.colorNeutralForeground3 : tokens.colorStatusDangerForeground1 }}
                  >
                    {record.params}
                  </p>
                </div>
                <Button
                  appearance="subtle"
                  size="small"
                  icon={<PlayRegular />}
                  onClick={() => replayMutation.mutate(record)}
                  disabled={replayMutation.isPending}
                  title="Replay"
                  aria-label={`Replay ${record.operation}`}
                />
              </div>
            ))
          )}
        </div>
      )}

      {replayMutation.isError && (
        <p
          className="text-xs text-center mt-2"
          style={{ color: tokens.colorStatusDangerForeground1 }}
        >
          Replay failed
        </p>
      )}
    </Card>
  );
};
