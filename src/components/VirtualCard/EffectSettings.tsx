import type { EffectSetting } from '../../bindings';
import { Box, Slider, Switch, Typography, FormControlLabel, FormControl, InputLabel, Select, MenuItem, Stack, Button } from '@mui/material';
import GradientPicker from '../GradientPicker/GradientPicker';

interface EffectSettingsProps {
  schema: EffectSetting[];
  settings: Record<string, any>;
  onSettingChange: (id: string, value: any) => void;
}

const getSortPriority = (setting: EffectSetting): number => {
  if (setting.id.includes('gradient')) return 1;
  if (setting.id.includes('color')) return 2;
  
  switch (setting.control.type) {
    case 'slider': return 3;
    case 'checkbox': return 4;
    case 'select': return 5;
    default: return 10; // Everything else comes last
  }
};

export function EffectSettings({ schema, settings, onSettingChange }: EffectSettingsProps) {
   const sortedSchema = [...schema].sort((a, b) => {
    const priorityA = getSortPriority(a);
    const priorityB = getSortPriority(b);
    return priorityA - priorityB; // This performs the comparison
  });

  console.log('Sorted Schema:', sortedSchema);
  return (

    <Box sx={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between' }}>
      {sortedSchema.map((setting, index) => {
        const value = settings[setting.id] ?? setting.defaultValue;

        switch (setting.control.type) {
          case 'slider':
            const { min, max, step } = setting.control;
            return (
              <Stack direction={'row'} key={setting.id} sx={{ mt: 2, flexBasis: '100%', justifyContent: 'space-between', alignItems: 'center' }}>
                <Typography variant='caption' gutterBottom sx={{ width: 100 }}>{setting.name}</Typography>
                <Slider
                  value={Number(value)}
                  onChange={(_e, newValue) => onSettingChange(setting.id, newValue)}
                  min={min}
                  max={max}
                  step={step}
                  valueLabelDisplay="auto"
                />
              </Stack>
            );
          case 'checkbox':
            return (
              <Button key={setting.id} sx={{ flexBasis: '48%', mt: 1 }} variant={!!value ? 'contained' : 'outlined'} onClick={() => onSettingChange(setting.id, !value)}>
                <Typography variant='caption'>{setting.name}</Typography>
              </Button>
            );
          case 'colorPicker':
            return (
              <GradientPicker
                pickerBgColor={String(value)}
                key={setting.id}
                title={setting.name}
                index={index}
                isGradient={true}
                // wrapperStyle={wrapperStyle}
                // colors={colors}
                // handleAddGradient={handleAddGradient}
                sendColorToVirtuals={(color: any) => {
                  onSettingChange(setting.id, color);
                }}
                // showHex={showHex}
              />
            );
          case 'select':
            const { options } = setting.control;
            return (
              <FormControl fullWidth size="small" key={setting.id} sx={{ mt: 2 }}>
                <InputLabel>{setting.name}</InputLabel>
                <Select
                  label={setting.name}
                  value={String(value)}
                  onChange={(e) => onSettingChange(setting.id, e.target.value)}
                >
                  {options.map(opt => (
                    <MenuItem key={opt} value={opt}>{opt}</MenuItem>
                  ))}
                </Select>
              </FormControl>
            );
          default:
            return null;
        }
      })}
    </Box>
  );
}