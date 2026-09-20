import type { FunctionalComponent } from 'preact';
import { useState } from 'preact/hooks';
import { useMutation } from '@tanstack/react-query';
import { Button, Card, Input, tokens } from '@fluentui/react-components';
import { api } from '../api';

interface InputControlsProps {
  deviceId: string;
}

export const InputControls: FunctionalComponent<InputControlsProps> = ({ deviceId }) => {
  const [text, setText] = useState('');
  const [gestureDuration, setGestureDuration] = useState('300');
  const [keyName, setKeyName] = useState('');
  const [keycode, setKeycode] = useState('');
  const [message, setMessage] = useState<string | null>(null);
  const textMutation = useMutation({
    mutationFn: () => api.sendText(deviceId, text),
    onSuccess: () => { setMessage('Text sent'); setText(''); },
    onError: (error) => setMessage(error.message),
  });
  const scrollMutation = useMutation({
    mutationFn: (direction: 'up' | 'down') => api.scroll(deviceId, direction, 2),
    onSuccess: () => setMessage('Scroll sent'),
    onError: (error) => setMessage(error.message),
  });
  const gestureMutation = useMutation({
    mutationFn: () => api.gesture(deviceId, [{ x: 200, y: 540 }, { x: 800, y: 540 }], Number(gestureDuration)),
    onSuccess: () => setMessage('Swipe sent'),
    onError: (error) => setMessage(error.message),
  });
  const keymapMutation = useMutation({
    mutationFn: () => api.sendMappedKey(deviceId, keyName.trim(), Number(keycode)),
    onSuccess: () => { setMessage('Mapped key sent'); setKeyName(''); setKeycode(''); },
    onError: (error) => setMessage(error.message),
  });

  return (
    <Card className="w-full" style={{ backgroundColor: tokens.colorNeutralBackground2 }}>
      <h3 className="text-sm font-semibold mb-3" style={{ color: tokens.colorBrandForeground1 }}>Input</h3>
      <div className="flex gap-2">
        <Input value={text} onChange={(_event, data) => setText(data.value)} placeholder="Send text" className="flex-1" size="small" />
        <Button appearance="primary" size="small" disabled={!text.trim() || textMutation.isPending} onClick={() => textMutation.mutate()}>Send</Button>
      </div>
      <div className="flex gap-2 mt-2">
        <Button appearance="outline" size="small" disabled={scrollMutation.isPending} onClick={() => scrollMutation.mutate('up')}>Scroll up</Button>
        <Button appearance="outline" size="small" disabled={scrollMutation.isPending} onClick={() => scrollMutation.mutate('down')}>Scroll down</Button>
      </div>
      <div className="flex gap-2 mt-2">
        <Input type="number" value={gestureDuration} onChange={(_event, data) => setGestureDuration(data.value)} aria-label="Swipe duration in milliseconds" className="flex-1" size="small" />
        <Button appearance="outline" size="small" disabled={gestureMutation.isPending || Number(gestureDuration) <= 0} onClick={() => gestureMutation.mutate()}>Swipe right</Button>
      </div>
      <div className="flex gap-2 mt-2">
        <Input value={keyName} onChange={(_event, data) => setKeyName(data.value)} placeholder="Custom key name" className="flex-1" size="small" />
        <Input type="number" value={keycode} onChange={(_event, data) => setKeycode(data.value)} placeholder="Keycode" className="w-24" size="small" />
        <Button appearance="outline" size="small" disabled={keymapMutation.isPending || !keyName.trim() || !keycode.trim() || !Number.isInteger(Number(keycode))} onClick={() => keymapMutation.mutate()}>Send</Button>
      </div>
      {message && <p className="text-xs mt-2" style={{ color: tokens.colorNeutralForeground2 }}>{message}</p>}
    </Card>
  );
};
