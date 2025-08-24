// src/components/DeviceCard/EffectPicker.tsx
import { memo } from 'react';
import { WledDevice } from '../../bindings';
import { MenuItem, Select } from '@mui/material';

interface EffectPickerProps {
  device: WledDevice;
  selectedEffect: string;
  onEffectSelect: (device: WledDevice, effectId: string) => void;
}

/**
 * Effect picker component for selecting an effect
 */
export const EffectPicker = memo(({
  device,
  selectedEffect,
  onEffectSelect,
}: EffectPickerProps) => {
  return (
    <Select
      sx={{ mt: 1.5 }}
      size='small'
      fullWidth
      value={selectedEffect}
      onChange={(e) => onEffectSelect(device, e.target.value)}
    >
      <MenuItem value="none" disabled>Choose Effect</MenuItem>
      <MenuItem value="rainbow">Rainbow</MenuItem>
      <MenuItem value="scroll">Scroll</MenuItem>
      <MenuItem value="scan">Scan</MenuItem>
      <MenuItem value="bladepower">BladePower</MenuItem>
    </Select> 
  );
});
