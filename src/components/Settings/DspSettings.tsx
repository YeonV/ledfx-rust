// src/components/Settings/DspSettings.tsx

import { useEffect, useState } from 'react';
import { useStore } from '../../store/useStore';
import { commands, type DspSettings as DspSettingsType } from '../../bindings';
import { Slider } from '@mui/material';
import { SettingsRow } from './SettingsRow';
import VolumeUpIcon from '@mui/icons-material/VolumeUp';
import ShutterSpeedIcon from '@mui/icons-material/ShutterSpeed';
import TimerIcon from '@mui/icons-material/Timer';

const DspSettings = () => {
  const { dspSettings } = useStore();
  // Use local state for sliders to avoid re-rendering the whole app on every drag
  const [localSettings, setLocalSettings] = useState<DspSettingsType | null>(dspSettings);

  // Sync local state when global state changes (e.g., from an import)
  useEffect(() => {
    setLocalSettings(dspSettings);
  }, [dspSettings]);

  // Debounce the backend call
  useEffect(() => {
    if (localSettings && JSON.stringify(localSettings) !== JSON.stringify(dspSettings)) {
      const handler = setTimeout(() => {
        commands.updateDspSettings(localSettings).catch(console.error);
      }, 500);
      return () => clearTimeout(handler);
    }
  }, [localSettings, dspSettings]);

  const handleSettingChange = (key: keyof DspSettingsType, value: number) => {
    if (localSettings) {
      setLocalSettings({ ...localSettings, [key]: value });
    }
  };

  if (!localSettings) {
    return null; // Or a loading indicator
  }

  return (
    <>
      <SettingsRow icon={<ShutterSpeedIcon />} title={`Smoothing: ${localSettings.smoothing_factor.toFixed(2)}`}>
        <Slider
          value={localSettings.smoothing_factor}
          onChange={(_e, val) => handleSettingChange('smoothing_factor', val as number)}
          min={0.01}
          max={0.99}
          step={0.01}
          valueLabelDisplay="auto"
        />
      </SettingsRow>
      <SettingsRow icon={<VolumeUpIcon />} title={`AGC Attack: ${localSettings.agc_attack.toFixed(3)}`}>
        <Slider
          value={localSettings.agc_attack}
          onChange={(_e, val) => handleSettingChange('agc_attack', val as number)}
          min={0.001}
          max={0.1}
          step={0.001}
          valueLabelDisplay="auto"
        />
      </SettingsRow>
      <SettingsRow icon={<VolumeUpIcon />} title={`AGC Decay: ${localSettings.agc_decay.toFixed(2)}`}>
        <Slider
          value={localSettings.agc_decay}
          onChange={(_e, val) => handleSettingChange('agc_decay', val as number)}
          min={0.01}
          max={0.5}
          step={0.01}
          valueLabelDisplay="auto"
        />
      </SettingsRow>
      <SettingsRow icon={<TimerIcon />} title={`Audio Delay: ${localSettings.audio_delay_ms}ms`}>
        <Slider
          value={localSettings.audio_delay_ms}
          onChange={(_e, val) => handleSettingChange('audio_delay_ms', val as number)}
          min={0}
          max={500}
          step={10}
          valueLabelDisplay="auto"
        />
      </SettingsRow>
    </>
  );
};

export default DspSettings;