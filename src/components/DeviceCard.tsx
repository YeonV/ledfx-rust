// src/components/DeviceCard.tsx

import React from 'react';
import { EffectPreview } from './EffectPreview';
import { EffectSetting, WledDevice } from '../bindings';

import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  Button, Card, CardActions, CardContent, CardHeader, Chip, FormControl,
  IconButton,
  InputLabel, MenuItem, Select, Stack, Tooltip, Typography
} from '@mui/material';
import LightbulbIcon from '@mui/icons-material/Lightbulb';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
import { ArrowDownward, ArrowDropDown, ExpandMore, Info, InfoOutline } from '@mui/icons-material';
import { EffectSettings } from './Effect.Settings';

interface DeviceCardProps {
  device: WledDevice;
  isActive: boolean;
  selectedEffect: string;
  onEffectSelect: (device: WledDevice, effectId: string) => void;
  onStart: (device: WledDevice) => void;
  onStop: (ip: string) => void;
  onSettingChange: (id: string, value: any) => void;
  schema?: EffectSetting[];
  settings?: Record<string, any>;
}

export const DeviceCard = React.memo(({
  device,
  isActive,
  selectedEffect,
  schema,
  settings,
  onEffectSelect,
  onStart,
  onStop,
  onSettingChange,
}: DeviceCardProps) => {
  return (
    <Card variant="outlined" sx={{ height: 'auto', width: '350px', display: 'flex', flexDirection: 'column' }}>
      <CardHeader
        sx={{ p: "8px 8px"}}
        avatar={<LightbulbIcon color={isActive ? 'warning' : 'inherit'} />}
        title={<Stack direction="row" alignItems="center">
          {device.name}
          <Tooltip sx={{ ml: 2}} title={<>
              <InfoLine label="Version" value={device.version} />
              <InfoLine label="LEDs" value={`${device.leds.count}`} />
              <InfoLine label="Chip" value={device.architecture} />
              <InfoLine label="IP" value={device.ip_address} />
          </>
          }>
            <span style={{ color: '#666', border: '1px solid #444', padding: '0px 8px', borderRadius: '4px', fontSize: '12px', marginLeft: 16 }}>{'INFO'}</span>
          </Tooltip></Stack>}          
        action={
        <IconButton
          onClick={() => isActive ? onStop(device.ip_address) : onStart(device)}
        >
          {isActive ? <StopIcon /> : <PlayArrowIcon />}
        </IconButton>
        }
      />
      <CardContent sx={{ p: '8px', pb: '8px !important' }}>
        <EffectPreview ipAddress={device.ip_address} active={isActive} />
          <Select
            sx={{ mt: 1.5 }}
            size='small'
            fullWidth
            value={selectedEffect}
            onChange={(e) => onEffectSelect(device, e.target.value)}
          >
            <MenuItem value="rainbow">Rainbow</MenuItem>
            <MenuItem value="scroll">Scroll</MenuItem>
            <MenuItem value="scan">Scan</MenuItem>
            <MenuItem value="bladepower">BladePower</MenuItem>
          </Select>        
        {schema && settings && <Accordion elevation={0} sx={{ mt: 1.5, border: '1px solid #444', borderRadius: '4px', overflow: 'hidden', minHeight: 40 }}>
          <AccordionSummary expandIcon={<ArrowDropDown />} sx={{ pr: 0.8}}>
            <Typography>Effect Settings</Typography>
          </AccordionSummary>
          <AccordionDetails>

            <EffectSettings schema={schema} settings={settings} onSettingChange={onSettingChange} />

          </AccordionDetails>
        </Accordion>}
      </CardContent>
    </Card>
  );
});

const InfoLine = ({ label, value }: { label: string; value: string }) => (
  <Stack direction={'row'} justifyContent={'space-between'} alignItems={'center'}>
    <Typography variant="body2" color="text.secondary" sx={{ mr: 2 }}>
      {label}:
    </Typography>
    <Typography variant="body2" color="text.primary">
      {value}
    </Typography>
  </Stack>
);