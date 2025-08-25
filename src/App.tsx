import { useEffect } from "react";
import { WledDiscoverer } from "./components/WledDiscoverer";
import { useFrameStore } from "./store/frameStore";
import { listen } from '@tauri-apps/api/event';
import { SettingsFab } from "./components/Settings/SettingsFab";
import { Virtuals as VirtualsComponent } from "./components/Virtuals";
import { MelbankVisualizerFab } from "./components/MelbankVisualizer/MelbankVisualizerFab";
import "./App.css";
import { commands, type Virtual, type Device } from "./bindings";
import { useStore } from "./store/useStore";
import { AddButton } from "./components/AddButton";

function App() {
  const { setAvailableEffects, setVirtuals, setDevices } = useStore();

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

    return () => {
      Promise.all([unlistenFrames, unlistenVirtuals, unlistenDevices]).then(([uf, uv, ud]) => {
        uf();
        uv();
        ud();
      });
    };
  }, [setAvailableEffects, setVirtuals, setDevices]);

  return (
    <main>
      <WledDiscoverer />
      <MelbankVisualizerFab />
      <SettingsFab />
      <VirtualsComponent />
      <AddButton />
    </main>
  );
}

export default App;