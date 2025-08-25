import { useEffect } from "react";
import { WledDiscoverer } from "./components/WledDiscoverer";
import { useFrameStore } from "./store/frameStore";
import { listen } from '@tauri-apps/api/event';
import { SettingsFab } from "./components/Settings/SettingsFab";
import { Virtuals as VirtualsComponent } from "./components/Virtuals"; // Renamed to avoid conflict
import { MelbankVisualizerFab } from "./components/MelbankVisualizer/MelbankVisualizerFab";
import "./App.css";
import { commands, type Virtual } from "./bindings"; // Import Virtual type
import { useStore } from "./store/useStore";

function App() {
  const { setAvailableEffects, setVirtuals } = useStore();

  useEffect(() => {
    // Fetch initial state once on startup
    const fetchInitialState = async () => {
      try {
        const effectsResult = await commands.getAvailableEffects();
        if (effectsResult.status === "ok") setAvailableEffects(effectsResult.data);

        const virtualsResult = await commands.getVirtuals();
        if (virtualsResult.status === 'ok') setVirtuals(virtualsResult.data);

      } catch (e) { console.error("Failed to fetch initial state:", e); }
    };
    fetchInitialState();

    // --- START: THE EVENT-DRIVEN FIX ---
    // Listen for frame data
    const unlistenFrames = listen<Record<string, number[]>>('engine-tick', (event) => {
      useFrameStore.setState({ frames: event.payload });
    });

    // Listen for changes to the list of virtuals
    const unlistenVirtuals = listen<Virtual[]>('virtuals-changed', (event) => {
      console.log("[EVENT] Virtuals list updated from backend:", event.payload);
      setVirtuals(event.payload);
    });
    // --- END: THE EVENT-DRIVEN FIX ---

    return () => {
      // Cleanup both listeners on unmount
      Promise.all([unlistenFrames, unlistenVirtuals]).then(([uf, uv]) => {
        uf();
        uv();
      });
    };
  }, [setAvailableEffects, setVirtuals]);

  return (
    <main>
      <WledDiscoverer />
      <MelbankVisualizerFab />
      <SettingsFab />
      <VirtualsComponent />
    </main>
  );
}

export default App;