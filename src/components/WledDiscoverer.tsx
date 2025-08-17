// src/components/WledDiscoverer.tsx

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { EffectPreview } from './EffectPreview';

// --- MUI Imports ---
import {
  Box,
  Button,
  Card,
  CardActions,
  CardContent,
  CardHeader,
  Grid,
  IconButton,
  LinearProgress,
  Stack,
  TextField,
  Typography,
  Alert
} from '@mui/material';
import { LoadingButton } from '@mui/lab';
import SearchIcon from '@mui/icons-material/Search';
import LightbulbIcon from '@mui/icons-material/Lightbulb';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';

// The interface remains the same.
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
  // The core logic remains identical.
  const [devices, setDevices] = useState<WledDevice[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [duration, setDuration] = useState(10);
  const [activeEffects, setActiveEffects] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const unlistenPromise = listen<WledDevice>('wled-device-found', (event) => {
      const foundDevice = event.payload;
      setDevices((prevDevices) => {
        if (!prevDevices.some(d => d.ip_address === foundDevice.ip_address)) {
          return [...prevDevices, foundDevice];
        }
        return prevDevices;
      });
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

  const handleToggleEffect = async (ip: string) => {
    try {
      // The command now returns a simple boolean.
      const isActive = await invoke<boolean>('toggle_ddp_effect', { ipAddress: ip });
      setActiveEffects(prev => ({ ...prev, [ip]: isActive }));
    } catch (err) {
      console.error("Failed to toggle DDP effect:", err);
    }
  };


  return (
    <Box sx={{ width: '100%', p: 2 }}>
      <Stack spacing={2} direction="row" alignItems="center" sx={{ mb: 2 }}>
        <TextField
          label="Scan Duration (s)"
          type="number"
          value={duration}
          onChange={(e) => setDuration(Number(e.target.value))}
          disabled={isScanning}
          size="small"
        />
        <LoadingButton
          onClick={handleDiscover}
          loading={isScanning}
          loadingPosition="start"
          startIcon={<SearchIcon />}
          variant="contained"
        >
          {isScanning ? 'Scanning...' : 'Discover'}
        </LoadingButton>
      </Stack>

      {isScanning && <LinearProgress sx={{ mb: 2 }} />}
      
      {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}

      <Grid container spacing={2}>
        {devices.map((device) => (
          <Grid size={{ xs: 12, sm: 6, md: 4 }} key={device.ip_address}>
            <Card variant="outlined">
              <CardHeader
                avatar={<IconButton><LightbulbIcon color={activeEffects[device.ip_address] ? 'warning' : 'inherit'} /></IconButton>}
                title={device.name}
                subheader={device.ip_address}
              />
              <CardContent>
                <EffectPreview ipAddress={device.ip_address} active={activeEffects[device.ip_address]} />
                <Typography variant="body2" color="text.secondary">
                  Version: {device.version}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  LEDs: {device.leds.count}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Architecture: {device.architecture}
                </Typography>
              </CardContent>
              <CardActions>
                <Button
                  size="small"
                  onClick={() => handleToggleEffect(device.ip_address)}
                  startIcon={activeEffects[device.ip_address] ? <StopIcon /> : <PlayArrowIcon />}
                  color={activeEffects[device.ip_address] ? 'secondary' : 'primary'}
                >
                  {activeEffects[device.ip_address] ? 'Stop Effect' : 'Start Effect'}
                </Button>
              </CardActions>
            </Card>
          </Grid>
        ))}
      </Grid>
    </Box>
  );
}