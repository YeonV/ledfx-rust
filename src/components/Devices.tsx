import { useCallback } from "react";
import { DeviceCard } from "./DeviceCard/DeviceCard";
import { commands } from "../bindings";
import { useStore } from "../store/useStore";
import { Grid } from "@mui/material";
import type { WledDevice, BladePowerLegacyConfig, EffectConfig, BaseEffectConfig, EffectSetting } from "../bindings";

const buildConfigPayload = (effectId: string, settings: Record<string, any>): EffectConfig | null => {
    const base: BaseEffectConfig = {
        mirror: !!settings.mirror,
        flip: !!settings.flip,
        blur: Number(settings.blur),
        background_color: String(settings.background_color),
    };

    if (effectId === 'bladepower') {
        return {
            mode: 'legacy',
            config: {
                decay: Number(settings.decay),
                multiplier: Number(settings.multiplier),
                frequency_range: String(settings.frequency_range),
                gradient: String(settings.gradient),
                ...base,
            }
        };
    }
    if (effectId === 'scan') {
        return {
            mode: 'scan',
            config: {
                speed: Number(settings.speed),
                width: Number(settings.width),
                gradient: String(settings.gradient),
                ...base,
            }
        };
    }
    return null;
};

export function Devices() {
  const {
    devices,
    activeEffects, setActiveEffects,
    selectedEffects, setSelectedEffects,
    effectSchemas, setEffectSchemas,
    effectSettings, setEffectSettings,
  } = useStore();

  const handleEffectSelection = useCallback(
    async (device: WledDevice, newEffectId: string) => {
      setSelectedEffects({ ...selectedEffects, [device.ip_address]: newEffectId });

      const deviceIp = device.ip_address;
      const effectAlreadyHasSettings = effectSettings[deviceIp]?.[newEffectId];

      if (!effectAlreadyHasSettings) {
        if (!effectSchemas[newEffectId]) {
          try {
            const result = await commands.getEffectSchema(newEffectId);
            if (result.status === "ok") {
              const schema = result.data;
              setEffectSchemas({ ...effectSchemas, [newEffectId]: schema });
              const defaultSettings = Object.fromEntries(schema.map((s: EffectSetting) => [s.id, s.defaultValue]));
              setEffectSettings({
                ...effectSettings,
                [deviceIp]: {
                  ...effectSettings[deviceIp],
                  [newEffectId]: defaultSettings,
                },
              });
              if (activeEffects[deviceIp]) {
                handleStartEffect(device, newEffectId, defaultSettings);
              }
            }
          } catch (e) { console.error(e); }
        }
      } else {
        if (activeEffects[deviceIp]) {
          handleStartEffect(device, newEffectId);
        }
      }
    },
    [activeEffects, effectSchemas, effectSettings, selectedEffects]
  );
  
  const handleSettingsChange = useCallback(
    (ip: string, id: string, value: any) => {
      const effectId = selectedEffects[ip];
      if (!effectId) return;

      const newSettingsForEffect = { ...effectSettings[ip]?.[effectId], [id]: value };
      const newSettings = {
        ...effectSettings,
        [ip]: {
          ...effectSettings[ip],
          [effectId]: newSettingsForEffect,
        }
      };
      setEffectSettings(newSettings);

      if (activeEffects[ip]) {
        const configPayload = buildConfigPayload(effectId, newSettingsForEffect);
        if (configPayload) {
            commands.updateEffectSettings(ip, configPayload).catch(console.error);
        }
      }
    },
    [activeEffects, effectSettings, selectedEffects]
  );

  const handleStartEffect = useCallback(
    async (device: WledDevice, effectIdOverride?: string, settingsOverride?: Record<string, any>) => {
      const effectId = effectIdOverride || selectedEffects[device.ip_address];
      const settings = settingsOverride || effectSettings[device.ip_address]?.[effectId];

      if (!effectId || !settings) { return; }
      
      const configPayload = buildConfigPayload(effectId, settings);
      
      try {
        await commands.startEffect(device.ip_address, device.leds.count, effectId, configPayload);
        setActiveEffects({ ...activeEffects, [device.ip_address]: true });
      } catch (err) { console.error("Failed to start effect:", err); }
    },
    [activeEffects, selectedEffects, effectSettings]
  );

  const handleStopEffect = useCallback(async (ip: string) => {
    try {
      await commands.stopEffect(ip);
      setActiveEffects({ ...activeEffects, [ip]: false });
    } catch (err) { console.error("Failed to stop effect:", err); }
  }, [activeEffects]);

  return (
    <Grid container spacing={2} sx={{p: 2}}>
        {devices.map((device) => {
            const effectId = selectedEffects[device.ip_address];
            return (
              // FIX: NO `item` PROP
              <Grid key={device.ip_address}>
                <DeviceCard
                  device={device}
                  isActive={activeEffects[device.ip_address] || false}
                  selectedEffect={effectId}
                  schema={effectSchemas[effectId]}
                  settings={effectSettings[device.ip_address]?.[effectId]}
                  onSettingChange={(id, value) => handleSettingsChange(device.ip_address, id, value)}
                  onEffectSelect={handleEffectSelection}
                  onStart={handleStartEffect}
                  onStop={handleStopEffect}
                />
              </Grid>
            )
        })}
    </Grid>
  );
}