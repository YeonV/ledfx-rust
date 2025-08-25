import { useEffect } from "react";
import { WledDiscoverer } from "./components/WledDiscoverer";
import { useFrameStore } from "./store/frameStore";
import { listen } from '@tauri-apps/api/event';
import { SettingsFab } from "./components/Settings/SettingsFab";
import { Devices } from "./components/Devices";
import { MelbankVisualizerFab } from "./components/MelbankVisualizer/MelbankVisualizerFab";
import "./App.css";
import { commands } from "./bindings";
import { useStore } from "./store/useStore";

function App() {
  const { setAvailableEffects } = useStore();

  useEffect(() => {
    const fetchAvailableEffects = async () => {
      try {
        const result = await commands.getAvailableEffects();
        if (result.status === "ok") {
          setAvailableEffects(result.data);
        } else {
          console.error("Failed to fetch available effects:", result.error);
        }
      } catch (e) { console.error(e); }
    };
    fetchAvailableEffects();

    const unlistenPromise = listen<Record<string, number[]>>('engine-tick', (event) => {
      useFrameStore.setState({ frames: event.payload });
    });
    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, [setAvailableEffects]);

  return (
    <main>
      <WledDiscoverer />
      <MelbankVisualizerFab />
      <SettingsFab />
      <Devices />
    </main>
  );
}

export default App;