import { memo } from 'react';
import { Virtual } from '../../bindings';
import { MenuItem, Select } from '@mui/material';
import { useStore } from '../../store/useStore';

interface EffectPickerProps {
  virtual: Virtual;
  selectedEffect: string;
  onEffectSelect: (virtual: Virtual, effectId: string) => void;
}

export const EffectPicker = memo(({
  virtual,
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
      onChange={(e) => onEffectSelect(virtual, e.target.value)}
      displayEmpty
    >
      <MenuItem value="" disabled>
        <em>Choose Effect</em>
      </MenuItem>
      {availableEffects.map(effect => (
        <MenuItem key={effect.id} value={effect.id}>{effect.name}</MenuItem>
      ))}
    </Select> 
  );
});