// src/components/WledDiscoverer.tsx

import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { DeviceCard } from "./DeviceCard";
import { commands } from "../bindings";
import type {
  WledDevice,
  AudioDevice,
  BladePowerLegacyConfig,
  BladePowerConfig,
  EffectConfig,
} from "../bindings";
import type { EffectSetting } from "../bindings";

import {
  Box,
  Grid,
  LinearProgress,
  Button,
  Stack,
  TextField,
  Alert,
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
  Drawer,
  IconButton,
  ToggleButtonGroup,
  ToggleButton,
  Paper,
} from "@mui/material";
import TrackChangesIcon from '@mui/icons-material/TrackChanges';
import SpeedIcon from '@mui/icons-material/Speed';
import SettingsIcon from "@mui/icons-material/Settings";
import Wled from "./icons/Wled";
import YZLogo2 from "./icons/YZ-Logo2";
import VolumeUpIcon from '@mui/icons-material/VolumeUp';

export function WledDiscoverer() {
  const [devices, setDevices] = useState<WledDevice[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [duration, setDuration] = useState(5);
  const [activeEffects, setActiveEffects] = useState<Record<string, boolean>>(
    {}
  );
  const [selectedEffects, setSelectedEffects] = useState<
    Record<string, string>
  >({});
  const [targetFps, setTargetFps] = useState(60);
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [selectedAudioDevice, setSelectedAudioDevice] = useState<string>("");
  const [engineMode, setEngineMode] = useState<"legacy" | "blade">("legacy");
  const [effectSchemas, setEffectSchemas] = useState<
    Record<string, EffectSetting[]>
  >({});
  const [effectSettings, setEffectSettings] = useState<
    Record<string, Record<string, any>>
  >({});
  const [openSettings, setOpenSettings] = useState(false);

  console.log(audioDevices)
  useEffect(() => {
    const setupAudio = async () => {
      const result = await commands.getAudioDevices();
      if (result.status === "ok") {
        setAudioDevices(result.data);
        if (result.data.length > 0) {
          const defaultDevice = result.data[0].name;
          setSelectedAudioDevice(defaultDevice);
          await commands.setAudioDevice(defaultDevice);
        }
      } else {
        setError(result.error);
      }
    };

    setupAudio().catch(console.error);

    const unlistenPromise = listen<WledDevice>("wled-device-found", (event) => {
      const foundDevice = event.payload;
      setDevices((prev) =>
        !prev.some((d) => d.ip_address === foundDevice.ip_address)
          ? [...prev, foundDevice]
          : prev
      );
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    const handler = setTimeout(() => {
      commands.setTargetFps(targetFps).catch(console.error);
    }, 500);
    return () => clearTimeout(handler);
  }, [targetFps]);

  const handleAudioDeviceChange = (event: SelectChangeEvent<string>) => {
    const deviceName = event.target.value;
    setSelectedAudioDevice(deviceName);
    commands.setAudioDevice(deviceName).catch(console.error);
  };

  const handleDiscover = async () => {
    setIsScanning(true);
    setError(null);
    setDevices([]);
    try {
      await commands.discoverWled(duration);
      setTimeout(() => setIsScanning(false), duration * 1000);
    } catch (err) {
      setError(err as string);
      setIsScanning(false);
    }
  };

  const handleEffectSelection = useCallback(
    async (device: WledDevice, newEffectId: string) => {
      setSelectedEffects((prev) => ({
        ...prev,
        [device.ip_address]: newEffectId,
      }));

      if (!effectSchemas[newEffectId]) {
        try {
          // For now, we assume only legacy schemas are available
          const result = await commands.getLegacyEffectSchema(newEffectId);
          if (result.status === "ok") {
            const schema = result.data;
            setEffectSchemas((prev) => ({ ...prev, [newEffectId]: schema }));
            const defaultSettings = Object.fromEntries(
              schema.map((s) => [s.id, s.defaultValue])
            );
            setEffectSettings((prev) => ({
              ...prev,
              [device.ip_address]: defaultSettings,
            }));
          }
        } catch (e) {
          console.error(e);
        }
      }

      if (activeEffects[device.ip_address]) {
        handleStartEffect(device, newEffectId);
      }
    },
    [activeEffects, effectSchemas]
  );

  const handleSettingsChange = useCallback(
    (ip: string, id: string, value: any) => {
      const newSettings = {
        ...effectSettings[ip],
        [id]: value,
      };
      setEffectSettings((prev) => ({
        ...prev,
        [ip]: newSettings,
      }));

      if (activeEffects[ip]) {
        // --- THE FIX: Build the payload in a type-safe way ---
        let configPayload: EffectConfig;
        if (engineMode === "legacy") {
          configPayload = {
            mode: engineMode,
            config: newSettings as BladePowerLegacyConfig,
          };
        } else {
          configPayload = {
            mode: "blade",
            config: newSettings as BladePowerConfig,
          };
        }
        commands.updateEffectSettings(ip, configPayload).catch(console.error);
      }
    },
    [activeEffects, effectSettings, engineMode]
  );

  const handleStartEffect = useCallback(
    async (device: WledDevice, effectIdOverride?: string) => {
      const effectId =
        effectIdOverride || selectedEffects[device.ip_address] || "bladepower";

      try {
        // --- THE FIX: Handle both cases ---
        if (effectId === "bladepower") {
          const settings = effectSettings[device.ip_address];
          if (!settings) {
            console.error("Settings not loaded for BladePower, cannot start.");
            return;
          }
          const configPayload = {
            mode: engineMode,
            config: settings,
          } as EffectConfig;
          await commands.startEffect(
            device.ip_address,
            device.leds.count,
            effectId,
            configPayload
          );
        } else {
          // For simple effects, send a null config.
          await commands.startEffect(
            device.ip_address,
            device.leds.count,
            effectId,
            null
          );
        }
        setActiveEffects((prev) => ({ ...prev, [device.ip_address]: true }));
      } catch (err) {
        console.error("Failed to start effect:", err);
      }
    },
    [selectedEffects, effectSettings, engineMode]
  );

  const handleStopEffect = useCallback(async (ip: string) => {
    try {
      await commands.stopEffect(ip);
      setActiveEffects((prev) => ({ ...prev, [ip]: false }));
    } catch (err) {
      console.error("Failed to stop effect:", err);
    }
  }, []);

  return (
    <Box sx={{ width: "100%", p: 2 }}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
        <Drawer
          open={openSettings}
          onClose={() => setOpenSettings(false)}
          anchor="right"
        >
          <Stack spacing={0}>
          <Card variant="outlined">
            <Stack direction="row" alignItems="center" spacing={2} p={2}>
              <SettingsIcon />
              <Typography variant="h6">Settings</Typography>
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
          <Card variant="outlined">
            <CardHeader
              avatar={<TrackChangesIcon />}
              title={`Scan Duration: ${duration}s`}
            />
            <CardContent>
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
            </CardContent>
          </Card>
          <Card variant="outlined">
            <CardHeader
              avatar={<SpeedIcon />}
              title={`Target: ${targetFps} FPS`}
            />
            <CardContent>
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
            </CardContent>
          </Card>
          <Card variant="outlined">
            <CardHeader
              avatar={<VolumeUpIcon />}
              title="Audio Settings"
              subheader="Select your audio input device"
            />
            <CardContent>
              <FormControl fullWidth size="small">
                <InputLabel>Audio Device</InputLabel>
                <Select
                  label="Audio Device"
                  value={selectedAudioDevice}
                  onChange={handleAudioDeviceChange}
                >
                  {audioDevices.map((device) => (
                    <MenuItem key={device.name} value={device.name} sx={{ justifyContent: "space-between", display: "flex" }}>
                      <Typography variant="body2" pr={2} display={'inline-flex'}>
                      {device.name.startsWith("System Audio") ? "🔊" : "🎤"} {device.name.replace('System Audio ', '').split(' (')[0].replace('(', '')} 
                      </Typography>
                      <Typography variant="caption" color="text.secondary"  display={'inline-flex'}>
                        {'(' + device.name.replace('System Audio ', '').split(' (')[1].replace('))', ')')}
                      </Typography>
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </CardContent>
          </Card>
          </Stack>
        </Drawer>
        <Button
          onClick={handleDiscover}
          loading={isScanning}
          loadingPosition="start"
          startIcon={<Wled />}
          variant="contained"
        >
          {isScanning ? "Scanning..." : "Discover"}
        </Button>
        <IconButton onClick={() => setOpenSettings(true)}>
          <SettingsIcon />
        </IconButton>
      </Stack>

      {isScanning && <LinearProgress sx={{ mb: 2 }} />}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <Grid container spacing={2}>
        {devices.map((device) => (
          <Grid key={device.ip_address}>
            <DeviceCard
              device={device}
              isActive={activeEffects[device.ip_address] || false}
              selectedEffect={
                selectedEffects[device.ip_address] || "bladepower"
              }
              // --- NEW: Pass schema and settings props ---
              schema={
                effectSchemas[
                selectedEffects[device.ip_address] || "bladepower"
                ]
              }
              settings={effectSettings[device.ip_address]}
              onSettingChange={(id: string, value: any) =>
                handleSettingsChange(device.ip_address, id, value)
              }
              onEffectSelect={handleEffectSelection}
              onStart={handleStartEffect}
              onStop={handleStopEffect}
            />
          </Grid>
        ))}
      </Grid>
    </Box>
  );
}
