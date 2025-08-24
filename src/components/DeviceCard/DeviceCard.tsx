import { memo } from 'react';
import { InfoLine } from './Infoline';
import { EffectPicker } from './EffectPicker';
import { EffectPreview } from './EffectPreview';
import { EffectSettings } from './EffectSettings';
import { Accordion, AccordionDetails, AccordionSummary, Card, CardContent, CardHeader, IconButton, Stack, Tooltip, Typography } from '@mui/material';
import { ArrowDropDown, Lightbulb as LightbulbIcon, PlayArrow as PlayArrowIcon, Stop as StopIcon} from '@mui/icons-material';
import { type EffectSetting, type WledDevice } from '../../bindings';

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

/**
 * Device card component for displaying and controlling a WLED device.
 */
export const DeviceCard = memo(({
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
        <EffectPicker device={device} selectedEffect={selectedEffect} onEffectSelect={onEffectSelect} />
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
