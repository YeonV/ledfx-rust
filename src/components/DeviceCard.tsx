// src/components/DeviceCard.tsx

import React from 'react';
import { EffectPreview } from './EffectPreview';
import { useFrameStore } from '../store/frameStore';
import { WledDevice } from '../types/wled'; // <-- IMPORT the central type

import {
  Button, Card, CardActions, CardContent, CardHeader, FormControl,
  IconButton, InputLabel, MenuItem, Select, Typography
} from '@mui/material';
import LightbulbIcon from '@mui/icons-material/Lightbulb';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';

// --- THE FIX: The props interface is now correct and clean ---
interface DeviceCardProps {
  device: WledDevice;
  isActive: boolean;
  selectedEffect: string;
  onEffectSelect: (device: WledDevice, effectId: string) => void;
  onStart: (device: WledDevice) => void;
  onStop: (ip: string) => void;
}

export const DeviceCard = React.memo(({
  device,
  isActive,
  selectedEffect,
  onEffectSelect,
  onStart,
  onStop,
}: DeviceCardProps) => {
  const pixels = useFrameStore(state => state.frames[device.ip_address]);

  return (
    <Card variant="outlined" sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <CardHeader
        avatar={<LightbulbIcon color={isActive ? 'warning' : 'inherit'} />}
        title={device.name}
        subheader={device.ip_address}
      />
      <CardContent sx={{ flexGrow: 1 }}>
        <EffectPreview ipAddress={device.ip_address} active={isActive} />
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
            onChange={(e) => onEffectSelect(device, e.target.value)}
          >
            <MenuItem value="rainbow">Rainbow</MenuItem>
            <MenuItem value="scroll">Scroll</MenuItem>
            <MenuItem value="scan">Scan</MenuItem>
            <MenuItem value="bladepower">BladePower</MenuItem>
          </Select>
        </FormControl>
        
        <Button
          size="small"
          onClick={() => isActive ? onStop(device.ip_address) : onStart(device)}
          startIcon={isActive ? <StopIcon /> : <PlayArrowIcon />}
          color={isActive ? 'secondary' : 'primary'}
        >
          {isActive ? 'Stop' : 'Start'}
        </Button>
      </CardActions>
    </Card>
  );
});