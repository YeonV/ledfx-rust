// src/components/WledDiscoverer.tsx

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { DeviceCard } from './DeviceCard';
import { WledDevice } from '../types/wled'; // <-- IMPORT the central type

import {
  Box, Grid, LinearProgress, Stack, TextField, Alert
} from '@mui/material';
import { LoadingButton } from '@mui/lab';
import SearchIcon from '@mui/icons-material/Search';

export function WledDiscoverer() {
  const [devices, setDevices] = useState<WledDevice[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [duration, setDuration] = useState(10);
  const [activeEffects, setActiveEffects] = useState<Record<string, boolean>>({});
  const [selectedEffects, setSelectedEffects] = useState<Record<string, string>>({});

  useEffect(() => {
    const unlistenPromise = listen<WledDevice>('wled-device-found', (event) => {
      const foundDevice = event.payload;
      setDevices((prev) => !prev.some(d => d.ip_address === foundDevice.ip_address) ? [...prev, foundDevice] : prev);
    });
    return () => { unlistenPromise.then(unlisten => unlisten()); };
  }, []);

  const handleDiscover = async () => {
    setIsScanning(true);
    setError(null);
    setDevices([]);
    try {
      await invoke('discover_wled', { durationSecs: duration });
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
      invoke('start_effect', {
        ipAddress: device.ip_address,
        ledCount: device.leds.count,
        effectId: newEffectId,
      }).catch(err => {
        console.error("Failed to switch effect:", err);
        setActiveEffects(prev => ({ ...prev, [device.ip_address]: false }));
      });
    }
  }, [activeEffects]);

  const handleStartEffect = useCallback(async (device: WledDevice) => {
    const effectId = selectedEffects[device.ip_address] || 'rainbow';
    try {
      await invoke('start_effect', {
        ipAddress: device.ip_address,
        ledCount: device.leds.count,
        effectId: effectId,
      });
      setActiveEffects(prev => ({ ...prev, [device.ip_address]: true }));
    } catch (err) { console.error("Failed to start effect:", err); }
  }, [selectedEffects]);

  const handleStopEffect = useCallback(async (ip: string) => {
    try {
      await invoke('stop_effect', { ipAddress: ip });
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