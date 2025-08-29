import { useEffect } from 'react';
import type { EffectSetting, PresetCollection } from '@/lib/rust';
import { Box, Slider, Typography, FormControl, InputLabel, Select, MenuItem, Stack, Button, Divider } from '@mui/material';
import GradientPicker from '../../base/GradientPicker/GradientPicker';
import { commands } from '@/lib/rust';
import { useStore } from '@/store/useStore';
import { PresetManager } from './PresetManager';

interface EffectSettingsProps {
  schema: EffectSetting[];
  settings: Record<string, any>;
  onSettingChange: (id: string, value: any) => void;
  effectId: string;
  onPresetLoad: (settings: any) => void;
  onPresetSave: (presetName: string) => void;
  onPresetDelete: (presetName: string) => void;
}

const getSortPriority = (setting: EffectSetting): number => {
  if (setting.id.includes('gradient')) return 1;
  if (setting.id.includes('color')) return 2;
  switch (setting.control.type) {
    case 'slider': return 3;
    case 'checkbox': return 4;
    case 'select': return 5;
    default: return 10;
  }
};

export function EffectSettings({ schema, settings, onSettingChange, effectId, onPresetLoad, onPresetSave, onPresetDelete }: EffectSettingsProps) {
  const { presetCache, setPresetsForEffect } = useStore();
  const presets = presetCache[effectId];

  useEffect(() => {
    if (effectId && !presets) {
        commands.loadPresets(effectId).then(result => {
            if (result.status === 'ok') {
                setPresetsForEffect(effectId, result.data as PresetCollection);
            }
        }).catch(console.error);
    }
  }, [effectId, presets, setPresetsForEffect]);

  const sortedSchema = [...schema].sort((a, b) => {
    const priorityA = getSortPriority(a);
    const priorityB = getSortPriority(b);
    return priorityA - priorityB;
  });

  return (
    <Box>
        <PresetManager
            presets={presets}
            settings={settings} // <-- Pass the current settings down
            onLoad={onPresetLoad}
            onSave={onPresetSave}
            onDelete={onPresetDelete}
        />
        <Box sx={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between', px: 1.5, mt: 1.5, border: '1px solid #444', borderRadius: '4px',  }}>
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
                <Button key={setting.id} sx={{ flexBasis: '49%', mt: 1 }} variant={!!value ? 'contained' : 'outlined'} onClick={() => onSettingChange(setting.id, !value)}>
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
                    sendColorToVirtuals={(color: any) => {
                    onSettingChange(setting.id, color);
                    }}
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
    </Box>
  );
}