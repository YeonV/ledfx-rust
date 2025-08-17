// src/components/controls/AudioSettings.tsx

import { useState, useEffect } from 'react';
import { Card, CardHeader, CardContent, FormControl, InputLabel, Select, MenuItem, SelectChangeEvent } from '@mui/material';
import SettingsIcon from '@mui/icons-material/Settings';
import { commands } from '../../bindings';
import type { AudioDevice } from '../../bindings';

export function AudioSettings() {
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [selectedAudioDevice, setSelectedAudioDevice] = useState<string>('');

  useEffect(() => {
    commands.getAudioDevices().then(result => {
      if (result.status === 'ok') {
        setAudioDevices(result.data);
        if (result.data.length > 0) {
          const defaultDevice = result.data[0].name;
          setSelectedAudioDevice(defaultDevice);
          commands.setAudioDevice(defaultDevice).catch(console.error);
        }
      } else {
        console.error(result.error);
      }
    });
  }, []);

  const handleAudioDeviceChange = (event: SelectChangeEvent<string>) => {
    const deviceName = event.target.value;
    setSelectedAudioDevice(deviceName);
    commands.setAudioDevice(deviceName).catch(console.error);
  };

  return (
    <Card variant="outlined" sx={{ mb: 2 }}>
      <CardHeader
        avatar={<SettingsIcon />}
        title="Audio Settings"
        subheader="Select your audio input device"
      />
      <CardContent>
        <FormControl fullWidth size="small">
          <InputLabel>Audio Device</InputLabel>
          <Select
            label="Audio Device"
            value={selectedAudioDevice}
            onChange={handleAudioDeviceChange}
          >
            {audioDevices.map((device) => (
              <MenuItem key={device.name} value={device.name}>
                {device.name}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
      </CardContent>
    </Card>
  );
}