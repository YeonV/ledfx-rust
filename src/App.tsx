// src/App.tsx

import { useEffect } from "react";
import { WledDiscoverer } from "./components/WledDiscoverer";
import { useFrameStore } from "./store/frameStore";
import { listen } from '@tauri-apps/api/event';
import { SettingsFab } from "./components/Settings/SettingsFab";
import { Devices } from "./components/Devices";
import { MelbankVisualizerFab } from "./components/MelbankVisualizer/MelbankVisualizerFab";
import "./App.css";

function App() {
  useEffect(() => {
    // console.log("Setting up global event listener for engine-tick...");
    const unlistenPromise = listen<Record<string, number[]>>('engine-tick', (event) => {
      useFrameStore.setState({ frames: event.payload });
    });
    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);

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