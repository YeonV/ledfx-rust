import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useStore } from '../store/useStore';
import { useFrameStore } from '../store/frameStore';
import type { Device, Virtual, PlaybackState, DspSettings, Scene, ActiveEffectsState } from '../bindings';

export const useEvents = () => {
    const { 
        setVirtuals, 
        setDevices, 
        setPlaybackState, 
        setScenes
    } = useStore();

    useEffect(() => {
        const unlistenFrames = listen<Record<string, number[]>>('engine-tick', (event) => {
          useFrameStore.setState({ frames: event.payload });
        });
        const unlistenVirtuals = listen<Virtual[]>('virtuals-changed', (event) => {
          setVirtuals(event.payload);
        });
        const unlistenDevices = listen<Device[]>('devices-changed', (event) => {
          setDevices(event.payload);
        });
        const unlistenPlayback = listen<PlaybackState>('playback-state-changed', (event) => {
          setPlaybackState(event.payload);
        });
        const unlistenDsp = listen<DspSettings>('dsp-settings-changed', (event) => {
          // Use setState to update both pieces of state at once
          useStore.setState({ dspSettings: event.payload, dirtyDspSettings: event.payload });
        });
        const unlistenScenes = listen<Scene[]>('scenes-changed', (event) => {
            setScenes(event.payload);
        });
        const unlistenSceneActivated = listen<ActiveEffectsState>('scene-activated', (event) => {
            const { active_scene_id, selected_effects, effect_settings, active_effects } = event.payload;
            const formattedSettings: Record<string, Record<string, any>> = {};
            for (const virtualId in effect_settings) {
                formattedSettings[virtualId] = {};
                for (const effectId in effect_settings[virtualId]) {
                    formattedSettings[virtualId][effectId] = effect_settings[virtualId][effectId]?.config;
                }
            }
        

        // Update the store in one go
        // Ensure all selectedEffects values are strings (not undefined)
        const filteredSelectedEffects: Record<string, string> = Object.fromEntries(
            Object.entries(selected_effects)
                .filter(([_, v]) => typeof v === "string")
                .map(([k, v]) => [k, v as string])
        );

        // Filter out undefined values to ensure type safety
        const filteredActiveEffects: Record<string, boolean> = Object.fromEntries(
            Object.entries(active_effects)
                .filter(([_, v]) => typeof v === "boolean")
                .map(([k, v]) => [k, v as boolean])
        );

            useStore.setState({
                activeSceneId: active_scene_id || null,
            selectedEffects: filteredSelectedEffects,
                effectSettings: formattedSettings,
            activeEffects: filteredActiveEffects,
            });
        });
    
        return () => {
          Promise.all([
              unlistenFrames, unlistenVirtuals, unlistenDevices, 
              unlistenPlayback, unlistenDsp, unlistenScenes, unlistenSceneActivated
            ]).then((unlisteners) => {
                unlisteners.forEach(unlisten => unlisten());
            });
        };
      }, [setVirtuals, setDevices, setPlaybackState, setScenes]);
};