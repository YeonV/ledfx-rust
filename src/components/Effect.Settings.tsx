// src/components/EffectSettings.tsx

import {
  Box, Slider, Switch, TextField, Typography, FormControlLabel,
  FormControl, InputLabel, Select, MenuItem
} from '@mui/material';
import type { EffectSetting } from '../bindings';
import ReactGPicker from 'react-gcolor-picker';
import GradientPicker from './GradientPicker/GradientPicker';

interface EffectSettingsProps {
  schema: EffectSetting[];
  settings: Record<string, any>;
  onSettingChange: (id: string, value: any) => void;
}

export function EffectSettings({ schema, settings, onSettingChange }: EffectSettingsProps) {
  const onChange = (value: any) => {
    console.log(value);
  };
  return (
    <Box>
      {schema.map((setting, index) => {
        // --- THE FIX: Use the defaultValue directly. It is already the primitive value. ---
        const value = settings[setting.id] ?? setting.defaultValue;

        // The `type` property is now guaranteed to be on the object.
        switch (setting.control.type) {
          case 'slider':
            const { min, max, step } = setting.control;
            return (
              <Box key={setting.id} sx={{ mt: 2 }}>
                <Typography gutterBottom>{setting.name}</Typography>
                <Slider
                  value={Number(value)}
                  onChange={(_e, newValue) => onSettingChange(setting.id, newValue)}
                  min={min}
                  max={max}
                  step={step}
                  valueLabelDisplay="auto"
                />
              </Box>
            );
          case 'checkbox':
            return (
              <FormControlLabel
                key={setting.id}
                control={
                  <Switch
                    checked={!!value}
                    onChange={(e) => onSettingChange(setting.id, e.target.checked)}
                  />
                }
                label={setting.name}
              />
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
                  value={String(value)} // Ensure value is a string
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