import { useEffect } from "react";
import { useFrameStore } from "./store/frameStore";
import { listen } from '@tauri-apps/api/event';
import { Virtuals } from "./components/Virtuals";
import { commands, type Virtual, type Device, PlaybackState, DspSettings, Scene, ActiveEffectsState } from "./bindings";
import { useStore } from "./store/useStore";
import { Alert, Box } from "@mui/material";
import { ConfigDrop } from "./components/ConfigDrop";
import TopBar from "./components/TopBar/TopBar";
import "./App.css";

function App() {
  const { 
    setAvailableEffects, 
    setVirtuals, 
    setDevices, 
    setPlaybackState, 
    setDspSettings,
    setScenes, // <-- Get the new setter
    virtuals, 
    error 
  } = useStore();

  useEffect(() => {
    const fetchInitialState = async () => {
      try {
        const effectsResult = await commands.getAvailableEffects();
        if (effectsResult.status === "ok") setAvailableEffects(effectsResult.data);

        const virtualsResult = await commands.getVirtuals();
        if (virtualsResult.status === 'ok') setVirtuals(virtualsResult.data);
        
        const devicesResult = await commands.getDevices();
        if (devicesResult.status === 'ok') setDevices(devicesResult.data);

        const dspResult = await commands.getDspSettings();
        if (dspResult.status === 'ok') setDspSettings(dspResult.data);
        
        // --- START: FETCH INITIAL SCENES ---
        const scenesResult = await commands.getScenes();
        if (scenesResult.status === 'ok') setScenes(scenesResult.data);
        // --- END: FETCH INITIAL SCENES ---

      } catch (e) { console.error("Failed to fetch initial state:", e); }
    };
    fetchInitialState();

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
      setDspSettings(event.payload);
    });
    
    // --- START: ADD SCENES LISTENER ---
    const unlistenScenes = listen<Scene[]>('scenes-changed', (event) => {
        setScenes(event.payload);
    });
    // --- END: ADD SCENES LISTENER ---
    const unlistenSceneActivated = listen<ActiveEffectsState>('scene-activated', (event) => {
        const { active_scene_id, selected_effects, effect_settings, active_effects } = event.payload;

        // The settings from Rust are EffectConfig, we need to extract the inner .config object
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
          unlistenFrames, 
          unlistenVirtuals, 
          unlistenDevices, 
          unlistenPlayback, 
          unlistenDsp,
          unlistenScenes,
          unlistenSceneActivated,
        ]).then(([uf, uv, ud, up, udsp, us, usa]) => {
        uf();
        uv();
        ud();
        up();
        udsp();
        us();
      });
    };
  }, [setAvailableEffects, setVirtuals, setDevices, setPlaybackState, setDspSettings, setScenes]); // <-- Add setter to dependency array

  return (
    // Box wrapper is a good practice for consistent layout
    <Box>
      <ConfigDrop />
      <TopBar />
      {error && (<Alert severity="error" sx={{ mt: 2, mb: 2 }}>{error}</Alert>)}
      <main style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 48px)', overflowY: 'auto' }}>
        {virtuals.length > 0 && <Virtuals />}     
      </main>
    </Box>
  );
}

export default App;