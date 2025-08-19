import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { DeviceCard } from './DeviceCard';
import { commands } from '../bindings';
import type { WledDevice, AudioDevice } from '../bindings';
import { invoke } from '@tauri-apps/api/core';

import {
  Box, Grid, LinearProgress, Stack, TextField, Alert, Slider, Typography,
  Card, CardHeader, FormControl, InputLabel, Select, MenuItem, SelectChangeEvent,
  CardContent
} from '@mui/material';
import { LoadingButton } from '@mui/lab';
import SearchIcon from '@mui/icons-material/Search';
import SettingsIcon from '@mui/icons-material/Settings';

export function WledDiscoverer() {
  const [devices, setDevices] = useState<WledDevice[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [duration, setDuration] = useState(10);
  const [activeEffects, setActiveEffects] = useState<Record<string, boolean>>({});
  const [selectedEffects, setSelectedEffects] = useState<Record<string, string>>({});
  const [targetFps, setTargetFps] = useState(60);
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [selectedAudioDevice, setSelectedAudioDevice] = useState<string>('');

  useEffect(() => {
    const setupAudio = async () => {
      if (window.__TAURI_METADATA__.__TAURI_PLATFORM__ === 'android') {
        try {
          await invoke('plugin:permissions|request_record_audio_permission');
          console.log("Audio permission granted or already available.");
        } catch (e) {
          setError(`Permission error: ${e}`);
          return;
        }
      }

      const result = await commands.getAudioDevices();
      if (result.status === 'ok') {
        setAudioDevices(result.data);
        if (result.data.length > 0) {
          const defaultDevice = result.data[0].name;
          setSelectedAudioDevice(defaultDevice);
          await commands.setAudioDevice(defaultDevice);
        }
      } else {
        setError(result.error);
      }
    };

    setupAudio().catch(console.error);

    const unlistenPromise = listen<WledDevice>('wled-device-found', (event) => {
      const foundDevice = event.payload;
      setDevices((prev) => !prev.some(d => d.ip_address === foundDevice.ip_address) ? [...prev, foundDevice] : prev);
    });
    return () => { unlistenPromise.then(unlisten => unlisten()); };
  }, []);

  useEffect(() => {
    const handler = setTimeout(() => {
      commands.setTargetFps(targetFps).catch(console.error);
    }, 500);
    return () => clearTimeout(handler);
  }, [targetFps]);

  const handleAudioDeviceChange = (event: SelectChangeEvent<string>) => {
    const deviceName = event.target.value;
    setSelectedAudioDevice(deviceName);
    commands.setAudioDevice(deviceName).catch(console.error);
  };

  const handleDiscover = async () => {
    setIsScanning(true);
    setError(null);
    setDevices([]);
    try {
      await commands.discoverWled(duration);
      setTimeout(() => setIsScanning(false), duration * 1000);
    } catch (err) {
      setError(err as string);
      setIsScanning(false);
    }
  };

  const handleEffectSelection = useCallback((device: WledDevice, newEffectId: string) => {
    setSelectedEffects(prev => ({ ...prev, [device.ip_address]: newEffectId }));
    const isCurrentlyActive = activeEffects[device.ip_address] || false;
    if (isCurrentlyActive) {
      commands.startEffect(device.ip_address, device.leds.count, newEffectId)
        .catch(err => {
          console.error("Failed to switch effect:", err);
          setActiveEffects(prev => ({ ...prev, [device.ip_address]: false }));
        });
    }
  }, [activeEffects]);

  const handleStartEffect = useCallback(async (device: WledDevice) => {
    const effectId = selectedEffects[device.ip_address] || 'rainbow';
    try {
      await commands.startEffect(device.ip_address, device.leds.count, effectId);
      setActiveEffects(prev => ({ ...prev, [device.ip_address]: true }));
    } catch (err) { console.error("Failed to start effect:", err); }
  }, [selectedEffects]);

  const handleStopEffect = useCallback(async (ip: string) => {
    try {
      await commands.stopEffect(ip);
      setActiveEffects(prev => ({ ...prev, [ip]: false }));
    } catch (err) { console.error("Failed to stop effect:", err); }
  }, []);

  return (
    <Box sx={{ width: '100%', p: 2 }}>
      <Stack spacing={2} direction="row" alignItems="center" sx={{ mb: 2 }}>
        <TextField
          label="Scan Duration (s)" type="number" value={duration}
          onChange={(e) => setDuration(Number(e.target.value))}
          disabled={isScanning} size="small"
        />
        <LoadingButton
          onClick={handleDiscover} loading={isScanning} loadingPosition="start"
          startIcon={<SearchIcon />} variant="contained"
        >
          {isScanning ? 'Scanning...' : 'Discover'}
        </LoadingButton>
      </Stack>

      <Box sx={{ width: 300, mb: 2 }}>
        <Typography gutterBottom>Target FPS: {targetFps}</Typography>
        <Slider
          value={targetFps}
          onChange={(e, newValue) => setTargetFps(newValue as number)}
          aria-labelledby="target-fps-slider"
          valueLabelDisplay="auto"
          step={5}
          marks
          min={10}
          max={120}
        />
      </Box>

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

      {isScanning && <LinearProgress sx={{ mb: 2 }} />}
      {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}

      <Grid container spacing={2}>
        {devices.map((device) => (
          <Grid size={{ xs: 12, sm: 6, md: 4 }} key={device.ip_address}>
            <DeviceCard
              device={device}
              isActive={activeEffects[device.ip_address] || false}
              selectedEffect={selectedEffects[device.ip_address] || 'rainbow'}
              onEffectSelect={handleEffectSelection}
              onStart={handleStartEffect}
              onStop={handleStopEffect}
            />
          </Grid>
        ))}
      </Grid>
    </Box>
  );
}
