import { useEffect } from "react";
import { WledDiscoverer } from "./components/WledDiscoverer";
import { useFrameStore } from "./store/frameStore";
import { listen } from '@tauri-apps/api/event';
import { Virtuals } from "./components/Virtuals";
import { commands, type Virtual, type Device, PlaybackState } from "./bindings";
import { useStore } from "./store/useStore";
import { AddButton } from "./components/AddButton";
import { GlobalControls } from "./components/GlobalControls";
import "./App.css";
import { AppBar, Box, Toolbar, Typography } from "@mui/material";

function App() {
  const { setAvailableEffects, setVirtuals, setDevices, setPlaybackState, devices, virtuals } = useStore();

  useEffect(() => {
    const fetchInitialState = async () => {
      try {
        const effectsResult = await commands.getAvailableEffects();
        if (effectsResult.status === "ok") setAvailableEffects(effectsResult.data);

        const virtualsResult = await commands.getVirtuals();
        if (virtualsResult.status === 'ok') setVirtuals(virtualsResult.data);
        
        const devicesResult = await commands.getDevices();
        if (devicesResult.status === 'ok') setDevices(devicesResult.data);

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

    return () => {
      Promise.all([unlistenFrames, unlistenVirtuals, unlistenDevices, unlistenPlayback]).then(([uf, uv, ud, up]) => {
        uf();
        uv();
        ud();
        up();
      });
    };
  }, [setAvailableEffects, setVirtuals, setDevices]);

  return (<>
    <AppBar elevation={0} color="error" position="sticky">
      <Toolbar color="error" sx={{ minHeight: '48px !important', justifyContent: 'space-between', px: '16px !important' }}>
        <Box>
          {devices.length > 0 && <WledDiscoverer variant='icon' />}
          {devices.length > 0 && <AddButton />}
        </Box>
        <GlobalControls />
      </Toolbar>
    </AppBar>
    <main style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 48px)', overflowY: 'auto' }}>
      {virtuals.length > 0 && <Virtuals />}
      {devices.length === 0 && <WledDiscoverer />}
     
    </main>
    </>
  );
}

export default App;