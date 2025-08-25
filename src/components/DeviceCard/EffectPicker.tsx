import { memo } from 'react';
import { WledDevice } from '../../bindings';
import { MenuItem, Select } from '@mui/material';
import { useStore } from '../../store/useStore';

interface EffectPickerProps {
  device: WledDevice;
  selectedEffect: string; // This prop can still be undefined from the parent
  onEffectSelect: (device: WledDevice, effectId: string) => void;
}

export const EffectPicker = memo(({
  device,
  selectedEffect,
  onEffectSelect,
}: EffectPickerProps) => {
  const { availableEffects } = useStore();

  const value = selectedEffect || '';

  return (
    <Select
      sx={{ mt: 1.5 }}
      size='small'
      fullWidth
      value={value}
      onChange={(e) => onEffectSelect(device, e.target.value)}
      displayEmpty
    >
      <MenuItem value="" disabled><em>Choose Effect</em></MenuItem>
      {availableEffects.map(effect => (
        <MenuItem key={effect.id} value={effect.id}>{effect.name}</MenuItem>
      ))}
    </Select> 
  );
});