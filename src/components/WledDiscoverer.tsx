// src/components/WledDiscoverer.tsx

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { EffectPreview } from './EffectPreview';

import {
  Box, Button, Card, CardActions, CardContent, CardHeader, Grid, IconButton,
  LinearProgress, Stack, TextField, Typography, Alert, Select, MenuItem,
  FormControl, InputLabel
} from '@mui/material';
import { LoadingButton } from '@mui/lab';
import SearchIcon from '@mui/icons-material/Search';
import LightbulbIcon from '@mui/icons-material/Lightbulb';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';

interface LedsInfo { count: number; }
interface MapInfo { id: number; }
interface WledDevice {
  ip_address: string;
  port: number;
  name: string;
  version: string;
  leds: LedsInfo;
  udp_port: number;
  architecture: string;
  maps: MapInfo[];
}

export function WledDiscoverer() {
  const [devices, setDevices] = useState<WledDevice[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [duration, setDuration] = useState(10);
  const [activeEffects, setActiveEffects] = useState<Record<string, boolean>>({});
  const [selectedEffects, setSelectedEffects] = useState<Record<string, string>>({});
  const [frameData, setFrameData] = useState<Record<string, number[]>>({});

  // Effect for mDNS discovery
  useEffect(() => {
    const unlistenPromise = listen<WledDevice>('wled-device-found', (event) => {
      const foundDevice = event.payload;
      setDevices((prev) => !prev.some(d => d.ip_address === foundDevice.ip_address) ? [...prev, foundDevice] : prev);
    });
    return () => { unlistenPromise.then(unlisten => unlisten()); };
  }, []);

  // Effect for the global animation loop
  useEffect(() => {
    let animationFrameId: number;
    const fetchFrames = async () => {
      try {
        const frames = await invoke<Record<string, number[]>>('get_latest_frames');
        setFrameData(frames);
      } catch (e) {
        console.error("Failed to fetch frames:", e);
      }
      animationFrameId = requestAnimationFrame(fetchFrames);
    };
    animationFrameId = requestAnimationFrame(fetchFrames);
    return () => cancelAnimationFrame(animationFrameId);
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

  const handleEffectSelection = (device: WledDevice, newEffectId: string) => {
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
  };

  const handleStartEffect = async (device: WledDevice) => {
    const effectId = selectedEffects[device.ip_address] || 'rainbow';
    try {
      await invoke('start_effect', {
        ipAddress: device.ip_address,
        ledCount: device.leds.count,
        effectId: effectId,
      });
      setActiveEffects(prev => ({ ...prev, [device.ip_address]: true }));
    } catch (err) {
      console.error("Failed to start effect:", err);
    }
  };

  const handleStopEffect = async (ip: string) => {
    try {
      await invoke('stop_effect', { ipAddress: ip });
      setActiveEffects(prev => ({ ...prev, [ip]: false }));
    } catch (err) {
      console.error("Failed to stop effect:", err);
    }
  };

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
        {devices.map((device) => {
          const isActive = activeEffects[device.ip_address] || false;
          const selectedEffect = selectedEffects[device.ip_address] || 'rainbow';

          return (
            <Grid size={{ xs: 12, sm: 6, md: 4 }} key={device.ip_address}>              <Card variant="outlined">
                <CardHeader
                  avatar={<IconButton><LightbulbIcon color={isActive ? 'warning' : 'inherit'} /></IconButton>}
                  title={device.name}
                  subheader={device.ip_address}
                />
                <CardContent>
                  <EffectPreview pixels={frameData[device.ip_address]} />
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                    Version: {device.version}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    LEDs: {device.leds.count}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Architecture: {device.architecture}
                  </Typography>
                </CardContent>
                <CardActions sx={{ justifyContent: 'space-between' }}>
                  <FormControl size="small" sx={{ minWidth: 120 }}>
                    <InputLabel>Effect</InputLabel>
                    <Select
                      value={selectedEffect}
                      label="Effect"
                      onChange={(e) => handleEffectSelection(device, e.target.value)}
                    >
                      <MenuItem value="rainbow">Rainbow</MenuItem>
                      <MenuItem value="scroll">Scroll</MenuItem>
                      <MenuItem value="scan">Scan</MenuItem>
                    </Select>
                  </FormControl>
                  
                  <Button
                    size="small"
                    onClick={() => isActive ? handleStopEffect(device.ip_address) : handleStartEffect(device)}
                    startIcon={isActive ? <StopIcon /> : <PlayArrowIcon />}
                    color={isActive ? 'secondary' : 'primary'}
                  >
                    {isActive ? 'Stop' : 'Start'}
                  </Button>
                </CardActions>
              </Card>
            </Grid>
          );
        })}
      </Grid>
    </Box>
  );
}