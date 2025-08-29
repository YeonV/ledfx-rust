import { useMemo, useEffect } from 'react';
import { useStore } from '@/store/useStore';
import { commands, type DspSettings as DspSettingsType, BladePlusParams } from '@/lib/rust';
import { Slider, Typography, Alert, Select, MenuItem, FormControl, InputLabel, Box, Button, Collapse, IconButton, Tooltip } from '@mui/material';
import { SettingsRow } from './SettingsRow';
import VolumeUpIcon from '@mui/icons-material/VolumeUp';
import ShutterSpeedIcon from '@mui/icons-material/ShutterSpeed';
import TimerIcon from '@mui/icons-material/Timer';
import TuneIcon from '@mui/icons-material/Tune';
import EqualizerIcon from '@mui/icons-material/Equalizer';
import GraphicEqIcon from '@mui/icons-material/GraphicEq';
import RestartAltIcon from '@mui/icons-material/RestartAlt';
import HubIcon from '@mui/icons-material/Hub';
import CurveVisualizer from './CurveVisualizer';

const deepEqual = (obj1: any, obj2: any) => JSON.stringify(obj1) === JSON.stringify(obj2);

const DspSettings = () => {
  const { dspSettings, dirtyDspSettings, setDirtyDspSettings } = useStore();

  const isDirty = useMemo(() => {
    if (!dspSettings || !dirtyDspSettings) return false;
    return dspSettings.fft_size !== dirtyDspSettings.fft_size ||
           dspSettings.num_bands !== dirtyDspSettings.num_bands ||
           dspSettings.min_freq !== dirtyDspSettings.min_freq ||
           dspSettings.max_freq !== dirtyDspSettings.max_freq ||
           !deepEqual(dspSettings.filterbank_type, dirtyDspSettings.filterbank_type) ||
           dspSettings.sample_rate !== dirtyDspSettings.sample_rate;
  }, [dspSettings, dirtyDspSettings]);

  const handleSettingChange = (key: keyof DspSettingsType, value: any) => {
    if (dirtyDspSettings) {
      const newSettings = { ...dirtyDspSettings, [key]: value };
      setDirtyDspSettings(newSettings);
    }
  };
  
  const handleBladePlusChange = (key: keyof BladePlusParams, value: number) => {
    if (dirtyDspSettings) {
        const newDirtySettings = JSON.parse(JSON.stringify(dirtyDspSettings));
        if (newDirtySettings.filterbank_type && typeof newDirtySettings.filterbank_type === 'object' && 'BladePlus' in newDirtySettings.filterbank_type) {
            newDirtySettings.filterbank_type.BladePlus[key] = value;
            setDirtyDspSettings(newDirtySettings);
        }
    }
  };

  useEffect(() => {
    if (!dirtyDspSettings || !dspSettings) return;
    const liveKeys: (keyof DspSettingsType)[] = ['smoothing_factor', 'agc_attack', 'agc_decay', 'audio_delay_ms'];
    const haveLiveSettingsChanged = liveKeys.some(key => dirtyDspSettings[key] !== dspSettings[key]);
    if (haveLiveSettingsChanged) {
        const handler = setTimeout(() => {
            commands.updateDspSettings(dirtyDspSettings).catch(console.error);
        }, 300);
        return () => clearTimeout(handler);
    }
  }, [dirtyDspSettings, dspSettings]);
  
  const handleApplyCriticalChanges = () => {
    if (!dirtyDspSettings) return;
    commands.updateDspSettings(dirtyDspSettings).then(() => {
        commands.restartAudioCapture();
    }).catch(console.error);
  };
  
  const handleResetToDefaults = async () => {
    const result = await commands.getDefaultEngineState();
    if (result.status === 'ok' && result.data.dsp_settings) {
        setDirtyDspSettings(result.data.dsp_settings);
    }
  };

  const handleResetSingle = (key: keyof DspSettingsType) => {
    if (dspSettings) {
        handleSettingChange(key, dspSettings[key]);
    }
  };
  
  // --- START: NEW DEDICATED HANDLER ---
  const handleResetBladePlusCurve = () => {
    if (dspSettings && dspSettings.blade_plus_params && dirtyDspSettings) {
        // Create a new object for the filterbank_type to ensure state update
        const newFilterbankType = { BladePlus: dspSettings.blade_plus_params };
        handleSettingChange('filterbank_type', newFilterbankType);
    }
  }
  // --- END: NEW DEDICATED HANDLER ---

  if (!dirtyDspSettings) return null;
  
  const getFilterbankTypeValue = (fb: DspSettingsType['filterbank_type']) => {
    if (typeof fb === 'string') return fb;
    if (fb && typeof fb === 'object' && 'BladePlus' in fb) return 'BladePlus';
    return 'Balanced';
  };

  const handleFilterbankChange = (newValue: string) => {
    let newFilterbankType: DspSettingsType['filterbank_type'];
    const defaultBladePlusParams = dspSettings?.blade_plus_params || { log_base: 12, multiplier: 3700, divisor: 230 };
    if (newValue === 'BladePlus') {
        newFilterbankType = { BladePlus: (typeof dirtyDspSettings.filterbank_type === 'object' && 'BladePlus' in dirtyDspSettings.filterbank_type && dirtyDspSettings.filterbank_type.BladePlus) || defaultBladePlusParams };
    } else {
        newFilterbankType = newValue as "Balanced" | "Precision" | "Vocal" | "Blade";
    }
    handleSettingChange('filterbank_type', newFilterbankType);
  }

  const currentBladePlusParams = (typeof dirtyDspSettings.filterbank_type === 'object' && 'BladePlus' in dirtyDspSettings.filterbank_type)
    ? dirtyDspSettings.filterbank_type.BladePlus
    : dspSettings?.blade_plus_params || { log_base: 12, multiplier: 3700, divisor: 230 };

  return (
    <>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', p: 2, pt: 0 }}>
        <Typography variant="h6">Advanced DSP Settings</Typography>
        <Button startIcon={<RestartAltIcon/>} onClick={handleResetToDefaults}>Reset All</Button>
      </Box>

      <Typography variant="subtitle1" sx={{ p: 2, pt: 0, pb: 1, fontWeight: 'bold' }}>Analysis</Typography>
        <SettingsRow icon={<TuneIcon />} title="FFT Size">
            <FormControl fullWidth size="small">
                <Select value={dirtyDspSettings.fft_size} onChange={(e) => handleSettingChange('fft_size', Number(e.target.value))}>
                    <MenuItem value={512}>512 (Fastest)</MenuItem>
                    <MenuItem value={1024}>1024 (Fast)</MenuItem>
                    <MenuItem value={2048}>2048 (Balanced)</MenuItem>
                    <MenuItem value={4096}>4096 (Detailed)</MenuItem>
                    <MenuItem value={8192}>8192 (High Precision)</MenuItem>
                </Select>
            </FormControl>
            <IconButton size="small" onClick={() => handleResetSingle('fft_size')}><RestartAltIcon/></IconButton>
        </SettingsRow>
        <SettingsRow icon={<EqualizerIcon />} title="Frequency Bands">
            <FormControl fullWidth size="small">
                <Select value={dirtyDspSettings.num_bands} onChange={(e) => handleSettingChange('num_bands', Number(e.target.value))}>
                    <MenuItem value={24}>24</MenuItem>
                    <MenuItem value={32}>32</MenuItem>
                    <MenuItem value={64}>64</MenuItem>
                    <MenuItem value={128}>128</MenuItem>
                    <MenuItem value={256}>256</MenuItem>
                </Select>
            </FormControl>
            <IconButton size="small" onClick={() => handleResetSingle('num_bands')}><RestartAltIcon/></IconButton>
        </SettingsRow>
        <SettingsRow icon={<HubIcon />} title="Sample Rate">
            <Tooltip title="Overrides the native device sample rate. 'Device Native' is recommended.">
                <FormControl fullWidth size="small">
                <Select value={dirtyDspSettings.sample_rate || 0} onChange={(e) => handleSettingChange('sample_rate', Number(e.target.value) === 0 ? null : Number(e.target.value))}>
                    <MenuItem value={0}>Device Native</MenuItem>
                    <MenuItem value={30000}>30000 Hz (LedFx Original)</MenuItem>
                    <MenuItem value={44100}>44100 Hz</MenuItem>
                    <MenuItem value={48000}>48000 Hz</MenuItem>
                </Select>
                </FormControl>
            </Tooltip>
            <IconButton size="small" onClick={() => handleResetSingle('sample_rate')}><RestartAltIcon/></IconButton>
        </SettingsRow>
      <SettingsRow icon={<GraphicEqIcon />} title="Filterbank Type">
        <FormControl fullWidth size="small">
          <Select value={getFilterbankTypeValue(dirtyDspSettings.filterbank_type)} onChange={(e) => handleFilterbankChange(e.target.value)}>
            <MenuItem value={"Balanced"}>Balanced</MenuItem>
            <MenuItem value={"Precision"}>Precision</MenuItem>
            <MenuItem value={"Vocal"}>Vocal</MenuItem>
            <MenuItem value={"Blade"}>Blade</MenuItem>
            <MenuItem value={"BladePlus"}>Custom (BladePlus)</MenuItem>
          </Select>
        </FormControl>
        <IconButton size="small" onClick={() => handleResetSingle('filterbank_type')}><RestartAltIcon/></IconButton>
      </SettingsRow>
      
      <Collapse in={getFilterbankTypeValue(dirtyDspSettings.filterbank_type) === 'BladePlus'}>
          <Box sx={{p: 2, border: '1px solid', borderColor: 'divider', m: 2, borderRadius: 1}}>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}>
                <Typography variant="h6" >BladePlus Curve Editor</Typography>
                {/* --- START: USE THE NEW HANDLER --- */}
                <Button size="small" startIcon={<RestartAltIcon/>} onClick={handleResetBladePlusCurve}>Reset Curve</Button>
                {/* --- END: USE THE NEW HANDLER --- */}
              </Box>
              <CurveVisualizer settings={dirtyDspSettings} />
              <SettingsRow title={`Curve Base: ${currentBladePlusParams.log_base.toFixed(1)}`}>
                <Slider value={currentBladePlusParams.log_base} onChange={(_e, v) => handleBladePlusChange('log_base', v as number)} min={2} max={20} step={0.1} />
              </SettingsRow>
              <SettingsRow title={`Low-End Focus: ${currentBladePlusParams.multiplier.toFixed(0)}`}>
                <Slider value={currentBladePlusParams.multiplier} onChange={(_e, v) => handleBladePlusChange('multiplier', v as number)} min={1000} max={5000} step={50} />
              </SettingsRow>
              <SettingsRow title={`Rolloff Point: ${currentBladePlusParams.divisor.toFixed(0)}`}>
                <Slider value={currentBladePlusParams.divisor} onChange={(_e, v) => handleBladePlusChange('divisor', v as number)} min={50} max={1000} step={10} />
              </SettingsRow>
          </Box>
      </Collapse>

      <SettingsRow icon={<VolumeUpIcon />} title={`Freq Range: ${dirtyDspSettings.min_freq}Hz - ${dirtyDspSettings.max_freq}Hz`}>
        <Slider value={[dirtyDspSettings.min_freq, dirtyDspSettings.max_freq]} onChange={(_e, val) => {
            const [min, max] = val as number[];
            setDirtyDspSettings({ ...dirtyDspSettings, min_freq: min, max_freq: max });
          }} min={20} max={22000} step={10} valueLabelDisplay="auto" disableSwap />
          <IconButton size="small" onClick={() => { handleResetSingle('min_freq'); handleResetSingle('max_freq'); }}><RestartAltIcon/></IconButton>
      </SettingsRow>

      <Typography variant="subtitle1" sx={{ p: 2, pb: 1, pt: 3, fontWeight: 'bold'}}>Smoothing & Gain</Typography>
      <SettingsRow icon={<ShutterSpeedIcon />} title={`Smoothing: ${dirtyDspSettings.smoothing_factor.toFixed(2)}`}>
        <Slider value={dirtyDspSettings.smoothing_factor} onChange={(_e, val) => handleSettingChange('smoothing_factor', val as number)} min={0.01} max={0.99} step={0.01} />
        <IconButton size="small" onClick={() => handleResetSingle('smoothing_factor')}><RestartAltIcon/></IconButton>
      </SettingsRow>
      <SettingsRow icon={<VolumeUpIcon />} title={`AGC Attack: ${dirtyDspSettings.agc_attack.toFixed(3)}`}>
        <Slider value={dirtyDspSettings.agc_attack} onChange={(_e, val) => handleSettingChange('agc_attack', val as number)} min={0.001} max={0.1} step={0.001} />
        <IconButton size="small" onClick={() => handleResetSingle('agc_attack')}><RestartAltIcon/></IconButton>
      </SettingsRow>
      <SettingsRow icon={<VolumeUpIcon />} title={`AGC Decay: ${dirtyDspSettings.agc_decay.toFixed(2)}`}>
        <Slider value={dirtyDspSettings.agc_decay} onChange={(_e, val) => handleSettingChange('agc_decay', val as number)} min={0.01} max={0.5} step={0.01} />
        <IconButton size="small" onClick={() => handleResetSingle('agc_decay')}><RestartAltIcon/></IconButton>
      </SettingsRow>
      <SettingsRow icon={<TimerIcon />} title={`Audio Delay: ${dirtyDspSettings.audio_delay_ms}ms`}>
        <Slider value={dirtyDspSettings.audio_delay_ms} onChange={(_e, val) => handleSettingChange('audio_delay_ms', val as number)} min={0} max={500} step={10} />
        <IconButton size="small" onClick={() => handleResetSingle('audio_delay_ms')}><RestartAltIcon/></IconButton>
      </SettingsRow>
      
      <Collapse in={isDirty}>
        <Box sx={{ p: 2, position: 'sticky', bottom: 0, backgroundColor: 'background.paper', borderTop: '1px solid', borderColor: 'divider' }}>
            <Alert severity='warning' sx={{ mb: 2 }}>You have unapplied critical settings.</Alert>
            <Button fullWidth variant="contained" color="warning" onClick={handleApplyCriticalChanges}>
                Apply & Restart Audio
            </Button>
        </Box>
      </Collapse>
    </>
  );
};

export default DspSettings;