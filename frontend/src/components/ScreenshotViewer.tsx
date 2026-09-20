import type { FunctionalComponent } from 'preact';
import { useState, useCallback, useEffect } from 'preact/hooks';
import { Button, Card, tokens, Spinner } from '@fluentui/react-components';
import { Camera24Regular, ArrowClockwiseRegular, DismissRegular } from '@fluentui/react-icons';
import { api } from '../api';

interface ScreenshotViewerProps {
  deviceId: string;
}

export const ScreenshotViewer: FunctionalComponent<ScreenshotViewerProps> = ({ deviceId }) => {
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [isCapturing, setIsCapturing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Cleanup blob URL on unmount
  useEffect(() => {
    return () => {
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
      }
    };
  }, [imageUrl]);

  const captureScreenshot = useCallback(async () => {
    setIsCapturing(true);
    setError(null);

    try {
      // Revoke previous URL to prevent memory leak
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
      }

      const blob = await api.takeScreenshot(deviceId);
      const url = URL.createObjectURL(blob);
      setImageUrl(url);
    } catch {
      setError('Failed to capture screenshot');
    } finally {
      setIsCapturing(false);
    }
  }, [deviceId, imageUrl]);

  const clearScreenshot = useCallback(() => {
    if (imageUrl) {
      URL.revokeObjectURL(imageUrl);
    }
    setImageUrl(null);
  }, [imageUrl]);

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
          <Camera24Regular style={{ color: tokens.colorBrandForeground1 }} />
          <h3
            className="text-sm font-semibold"
            style={{ color: tokens.colorBrandForeground1 }}
          >
            Screenshot
          </h3>
        </div>
        <div className="flex gap-1">
          {imageUrl && (
            <Button
              appearance="subtle"
              size="small"
              icon={<DismissRegular />}
              onClick={clearScreenshot}
              aria-label="Clear screenshot"
            />
          )}
          <Button
            appearance="subtle"
            size="small"
            icon={isCapturing ? <Spinner size="tiny" /> : <ArrowClockwiseRegular />}
            onClick={captureScreenshot}
            disabled={isCapturing}
            aria-label="Capture screenshot"
          >
            {isCapturing ? 'Capturing...' : 'Capture'}
          </Button>
        </div>
      </div>

      {error && (
        <p
          className="text-xs text-center py-2"
          style={{ color: tokens.colorStatusDangerForeground1 }}
        >
          {error}
        </p>
      )}

      {imageUrl ? (
        <div
          className="rounded-lg overflow-hidden border"
          style={{ borderColor: tokens.colorNeutralStroke2 }}
        >
          <img
            src={imageUrl}
            alt="Device screenshot"
            className="w-full h-auto"
            style={{ maxHeight: '400px', objectFit: 'contain' }}
          />
        </div>
      ) : (
        !isCapturing && !error && (
          <div
            className="flex flex-col items-center justify-center py-8 rounded-lg"
            style={{ backgroundColor: tokens.colorNeutralBackground1 }}
          >
            <Camera24Regular
              className="mb-2"
              style={{ color: tokens.colorNeutralForeground3 }}
            />
            <p
              className="text-xs"
              style={{ color: tokens.colorNeutralForeground2 }}
            >
              Click Capture to take a screenshot
            </p>
          </div>
        )
      )}
    </Card>
  );
};
