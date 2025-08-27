import { useEffect, useState } from 'react';
import { commands, type DspSettings } from '../../bindings';
import { SettingsRow } from './SettingsRow';
import { Slider } from '@mui/material';
import { AirlineStops, AutoGraph, SouthEast } from '@mui/icons-material';

const DspSettings = () => {
  const [settings, setSettings] = useState<DspSettings | null>(null);

  useEffect(() => {
    const fetchSettings = async () => {
      try {
        const result = await commands.getDspSettings();
        if (result.status === "ok") {
          setSettings(result.data);
        }
      } catch (e) { console.error(e); }
    };
    fetchSettings();
  }, []);

  const handleSettingChange = (field: keyof DspSettings, value: number) => {
    if (!settings) return;
    const newSettings = { ...settings, [field]: value };
    setSettings(newSettings);
    commands.updateDspSettings(newSettings);
  };

  if (!settings) {
    return <div>Loading DSP Settings...</div>;
  }

  return (<>

    <SettingsRow icon={<AutoGraph />} title={`Smoothing Factor: ${settings.smoothing_factor.toFixed(2)}`}>
      <Slider
          value={settings.smoothing_factor}
          onChange={(_e, newValue) => handleSettingChange('smoothing_factor', newValue as number)}
          aria-labelledby="smoothing-factor-slider"
          valueLabelDisplay="auto"
          step={0.01}
          min={0}
          max={0.99}
      />
    </SettingsRow>
    <SettingsRow icon={<AirlineStops />} title={`AGC Attack: ${settings.agc_attack.toFixed(3)}`}>
      <Slider
          value={settings.agc_attack}
          onChange={(_e, newValue) => handleSettingChange('agc_attack', newValue as number)}
          aria-labelledby="agc-attack-slider"
          valueLabelDisplay="auto"
          step={0.001}
          min={0.001}
          max={0.1}
      />
    </SettingsRow>
    <SettingsRow icon={<SouthEast />} title={`AGC Decay: ${settings.agc_decay.toFixed(3)}`}>
      <Slider
          value={settings.agc_decay}
          onChange={(_e, newValue) => handleSettingChange('agc_decay', newValue as number)}
          aria-labelledby="agc-decay-slider"
          valueLabelDisplay="auto"
          step={0.001}
          min={0.001}
          max={0.2}
      />
    </SettingsRow>
    <SettingsRow icon={<SouthEast />} title={`Audio Delay (ms): ${settings.audio_delay_ms}`}>
        <Slider
          value={settings.audio_delay_ms}
          onChange={(_e, newValue) => handleSettingChange('audio_delay_ms', newValue as number)}
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