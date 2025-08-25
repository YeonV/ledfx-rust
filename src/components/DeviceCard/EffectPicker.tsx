import { memo } from 'react';
import { WledDevice } from '../../bindings';
import { MenuItem, Select } from '@mui/material';
import { useStore } from '../../store/useStore';

interface EffectPickerProps {
  device: WledDevice;
  selectedEffect: string;
  onEffectSelect: (device: WledDevice, effectId: string) => void;
}

export const EffectPicker = memo(({
  device,
  selectedEffect,
  onEffectSelect,
}: EffectPickerProps) => {
  const { availableEffects } = useStore();

  return (
    <Select
      sx={{ mt: 1.5 }}
      size='small'
      fullWidth
      value={selectedEffect}
      onChange={(e) => onEffectSelect(device, e.target.value)}
    >
      <MenuItem value={undefined} disabled>Choose Effect</MenuItem>
      {availableEffects.map(effect => (
        <MenuItem key={effect.id} value={effect.id}>{effect.name}</MenuItem>
      ))}
    </Select> 
  );
});