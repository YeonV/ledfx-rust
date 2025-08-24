// src/components/Settings.tsx

import { useEffect } from "react";
import { commands } from "../../bindings";
import {
    Stack,
    Slider,
    Typography,
    Card,
    CardHeader,
    FormControl,
    InputLabel,
    Select,
    MenuItem,
    SelectChangeEvent,
    CardContent,
    ToggleButtonGroup,
    ToggleButton,
    IconButton,
    Accordion,
    AccordionSummary,
    AccordionDetails,
} from "@mui/material";
import TrackChangesIcon from '@mui/icons-material/TrackChanges';
import SpeedIcon from '@mui/icons-material/Speed';
import SettingsIcon from "@mui/icons-material/Settings";
import YZLogo2 from "../Icons/YZ-Logo2";
import VolumeUpIcon from '@mui/icons-material/VolumeUp';
import { useStore } from "../../store/useStore";
import DspSettings from "./DspSettings";
import { ArrowDropDown, Close, SettingsSuggest } from "@mui/icons-material";
import { SettingsRow } from "./SettingsRow";

export function Settings() {
  const {
    duration, setDuration,
    targetFps, setTargetFps,
    audioDevices,
    selectedAudioDevice, setSelectedAudioDevice,
    engineMode, setEngineMode, setOpenSettings
  } = useStore();

    const handleAudioDeviceChange = (event: SelectChangeEvent<string>) => {
        const deviceName = event.target.value;
        setSelectedAudioDevice(deviceName);
        commands.setAudioDevice(deviceName).catch(console.error);
    };

    
    useEffect(() => {
        const handler = setTimeout(() => {
        commands.setTargetFps(targetFps).catch(console.error);
        }, 500);
        return () => clearTimeout(handler);
    }, [targetFps]);

    return (
        <Stack spacing={0}>
            <Card variant="outlined">
                <Stack direction="row" alignItems="center" justifyContent={'space-between'} p={2}>
                <Stack direction="row" alignItems="center" spacing={2}>
                    <SettingsIcon />                    
                    <Typography variant="h6">Settings</Typography>
                </Stack>
                <IconButton onClick={() => setOpenSettings(false)}>
                    <Close />
                </IconButton>
            </Stack>
            </Card>
            <Card variant="outlined">
                <CardContent>
                    <Stack direction={'row'} justifyContent="space-between" alignItems="center">
                        <Stack direction="row" alignItems="center" spacing={1}>
                            <YZLogo2 />
                            <Typography variant="body2" pr={2}>Engine Mode:</Typography>
                        </Stack>
                        <ToggleButtonGroup
                            color="primary"
                            value={engineMode}
                            exclusive
                            onChange={(_event, newAlignment) => setEngineMode(newAlignment)}
                        >
                            <ToggleButton value="legacy">Legacy</ToggleButton>
                            <ToggleButton value="blade">Blade</ToggleButton>
                        </ToggleButtonGroup>
                    </Stack>
                </CardContent>
            </Card>
            <SettingsRow icon={<TrackChangesIcon />} title={`Scan Duration: ${duration}s`}>
                <Slider
                    value={duration}
                    onChange={(_e, newValue) => setDuration(Number(newValue))}
                    aria-labelledby="duration-slider"
                    valueLabelDisplay="auto"
                    step={1}
                    marks
                    min={1}
                    max={30}
                />
            </SettingsRow>
            <SettingsRow icon={<SpeedIcon />} title={`Target: ${targetFps} FPS`}>
                <Slider
                    value={targetFps}
                    onChange={(_e, newValue) => setTargetFps(newValue as number)}
                    aria-labelledby="target-fps-slider"
                    valueLabelDisplay="auto"
                    step={5}
                    marks
                    min={10}
                    max={120}
                />
            </SettingsRow>
            <SettingsRow icon={<VolumeUpIcon />} title={'Audio Input'}>
                <Select
                disableUnderline
                variant="standard"
                    label="Audio Device"
                    value={selectedAudioDevice}
                    onChange={handleAudioDeviceChange}
                >
                    {audioDevices.map((device) => (
                        <MenuItem key={device.name} value={device.name} sx={{ justifyContent: "space-between", display: "flex" }}>
                            <Typography variant="body2" pr={2} display={'inline-flex'}>
                                {device.name.startsWith("System Audio") ? "🔊" : "🎤"} {device.name.replace('System Audio ', '').split(' (')[0].replace('(', '')}
                            </Typography>
                            <Typography variant="caption" color="text.secondary" display={'inline-flex'}>
                                {'(' + device.name.replace('System Audio ', '').split(' (')[1].replace('))', ')')}
                            </Typography>
                        </MenuItem>
                    ))}
                </Select>
            </SettingsRow>
            <Accordion elevation={0}>
                <AccordionSummary expandIcon={<ArrowDropDown />}>
                    <SettingsSuggest sx={{ mr: 2 }} />
                    <Typography variant="h6">Advanced Settings</Typography>
                </AccordionSummary>
                <AccordionDetails sx={{ p: 0 }}>
                    <DspSettings />
                </AccordionDetails>
            </Accordion>
        </Stack>
    );
}
