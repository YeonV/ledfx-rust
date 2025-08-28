import { useEffect } from "react";
import { useFrameStore } from "./store/frameStore";
import { listen } from '@tauri-apps/api/event';
import { Virtuals } from "./components/Virtuals";
import { commands, type Virtual, type Device, PlaybackState, DspSettings } from "./bindings";
import { useStore } from "./store/useStore";
import { Alert } from "@mui/material";
import { ConfigDrop } from "./components/ConfigDrop";
import TopBar from "./components/TopBar/TopBar";
import "./App.css";

function App() {
  const { setAvailableEffects, setVirtuals, setDevices, setPlaybackState, setDspSettings, virtuals, error } = useStore();

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

    return () => {
      Promise.all([unlistenFrames, unlistenVirtuals, unlistenDevices, unlistenPlayback, unlistenDsp]).then(([uf, uv, ud, up, udsp]) => {
        uf();
        uv();
        ud();
        up();
      });
    };
  }, [setAvailableEffects, setVirtuals, setDevices]);

  return (
    <>
      <ConfigDrop />
      <TopBar />
      {error && (<Alert severity="error" sx={{ mt: 2, mb: 2 }}>{error}</Alert>)}
      <main style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 48px)', overflowY: 'auto' }}>
        {virtuals.length > 0 && <Virtuals />}     
      </main>
    </>
  );
}

export default App;