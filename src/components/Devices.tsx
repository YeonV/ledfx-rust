// src/components/WledDiscoverer.tsx

import { useCallback } from "react";
import { DeviceCard } from "./DeviceCard/DeviceCard";
import { commands } from "../bindings";
import { useStore } from "../store/useStore";
import { Grid } from "@mui/material";
import type { WledDevice, BladePowerLegacyConfig, BladePowerConfig, EffectConfig } from "../bindings";

/**
 * Device management component for controlling WLED devices.
 */
export function Devices() {
  const {
    devices,
    activeEffects, setActiveEffects,
    selectedEffects, setSelectedEffects,
    engineMode,
    effectSchemas, setEffectSchemas,
    effectSettings, setEffectSettings,
  } = useStore();

  console.log(devices)
  const handleEffectSelection = useCallback(
    async (device: WledDevice, newEffectId: string) => {
      setSelectedEffects({
        ...selectedEffects,
        [device.ip_address]: newEffectId,
      });

      if (!effectSchemas[newEffectId]) {
        try {
          const result = await commands.getLegacyEffectSchema(newEffectId);
          if (result.status === "ok") {
            const schema = result.data;
            setEffectSchemas({ ...effectSchemas, [newEffectId]: schema });
            const defaultSettings = Object.fromEntries(
              schema.map((s) => [s.id, s.defaultValue])
            );
            setEffectSettings({ ...effectSettings, [device.ip_address]: defaultSettings });
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
      setEffectSettings({ ...effectSettings, [ip]: newSettings });

      if (activeEffects[ip]) {
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
          await commands.startEffect(
            device.ip_address,
            device.leds.count,
            effectId,
            null
          );
        }
        setActiveEffects({ ...activeEffects, [device.ip_address]: true });
      } catch (err) {
        console.error("Failed to start effect:", err);
      }
    },
    [selectedEffects, effectSettings, engineMode]
  );

  const handleStopEffect = useCallback(async (ip: string) => {
    try {
      await commands.stopEffect(ip);
      setActiveEffects({ ...activeEffects, [ip]: false });
    } catch (err) {
      console.error("Failed to stop effect:", err);
    }
  }, []);

  return (
    <Grid container spacing={2}>
        {devices.map((device) => (
          <Grid key={device.ip_address}>
            <DeviceCard
              device={device}
              isActive={activeEffects[device.ip_address] || false}
              selectedEffect={selectedEffects[device.ip_address] || "none"}
              schema={effectSchemas[selectedEffects[device.ip_address] || "bladepower"]}
              settings={effectSettings[device.ip_address]}
              onSettingChange={(id: string, value: any) => handleSettingsChange(device.ip_address, id, value)}
              onEffectSelect={handleEffectSelection}
              onStart={handleStartEffect}
              onStop={handleStopEffect}
            />
          </Grid>
        ))}
    </Grid>
  );
}
