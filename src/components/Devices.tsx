import { useCallback } from "react";
import { DeviceCard } from "./DeviceCard/DeviceCard";
import { commands } from "../bindings";
import { useStore } from "../store/useStore";
import { Grid } from "@mui/material";
import type { WledDevice, EffectConfig, EffectSetting, EffectInfo } from "../bindings";

const buildConfigPayload = (effectId: string, settings: Record<string, any>): EffectConfig | null => {
    if (!effectId || effectId === 'none') return null;
    return {
      type: effectId,
      config: settings,
    } as EffectConfig;
};

export function Devices() {
  const {
    devices,
    activeEffects, setActiveEffects,
    selectedEffects, setSelectedEffects,
    effectSchemas, setEffectSchemas,
    effectSettings, setEffectSettings
  } = useStore();

  const handleEffectSelection = useCallback(
    async (device: WledDevice, newEffectId: string) => {
      const deviceIp = device.ip_address;

      setSelectedEffects({ ...selectedEffects, [deviceIp]: newEffectId });

      let schema = effectSchemas[newEffectId];
      if (!schema) {
        try {
          const result = await commands.getEffectSchema(newEffectId);
          if (result.status === "ok") {
            schema = result.data;
            setEffectSchemas({ ...effectSchemas, [newEffectId]: schema });
          } else { return; }
        } catch (e) { return; }
      }
      
      const effectAlreadyHasSettings = effectSettings[deviceIp]?.[newEffectId];
      if (!effectAlreadyHasSettings && schema) {
        const defaultSettings = Object.fromEntries(
            schema.map((s: EffectSetting) => [s.id, s.defaultValue])
        );
        const newSettings = { ...effectSettings, [deviceIp]: { ...effectSettings[deviceIp], [newEffectId]: defaultSettings } };
        setEffectSettings(newSettings);

        if (activeEffects[deviceIp]) {
          handleStartEffect(device, newEffectId, defaultSettings);
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
      const newSettings = { ...effectSettings, [ip]: { ...effectSettings[ip], [effectId]: newSettingsForEffect } };
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
      // console.log(`[FRONTEND LOG] 1. handleStartEffect called for ${device.ip_address}`);
      const effectId = effectIdOverride || selectedEffects[device.ip_address];
      const settings = settingsOverride || effectSettings[device.ip_address]?.[effectId];
      
      if (!effectId || !settings) {
        console.error(`[FRONTEND LOG] ERROR: Missing effectId or settings. Cannot start.`);
        return;
      }
      
      const configPayload = buildConfigPayload(effectId, settings);
      // console.log('[FRONTEND LOG] 2. Built config payload:', JSON.stringify(configPayload, null, 2));
      
      if (configPayload) {
        try {
          // console.log('[FRONTEND LOG] 3. Calling backend command: commands.startEffect');
          await commands.startEffect(device.ip_address, device.leds.count, configPayload);
          // console.log('[FRONTEND LOG] 4. Backend command successful.');
          setActiveEffects({ ...activeEffects, [device.ip_address]: true });
        } catch (err) {
          console.error("[FRONTEND LOG] ERROR: Backend command 'startEffect' failed:", err);
        }
      } else {
        console.error("[FRONTEND LOG] ERROR: configPayload was null. Cannot start.");
      }
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
              <Grid key={device.ip_address}>
                <DeviceCard
                  device={device}
                  isActive={activeEffects[device.ip_address] || false}
                  selectedEffect={effectId}
                  schema={effectSchemas[effectId]}
                  settings={effectSettings[device.ip_address]?.[effectId]}
                  onSettingChange={(id, value) => handleSettingsChange(device.ip_address, id, value)}
                  onEffectSelect={handleEffectSelection}
                  onStart={() => handleStartEffect(device)}
                  onStop={() => handleStopEffect(device.ip_address)}
                />
              </Grid>
            )
        })}
    </Grid>
  );
}