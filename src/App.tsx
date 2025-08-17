// src/App.tsx

import { useEffect } from "react";
import { WledDiscoverer } from "./components/WledDiscoverer";
import { useFrameStore } from "./store/frameStore";
import { listen } from '@tauri-apps/api/event';
import "./App.css";

function App() {
  // --- THE FIX: The global event listener is started here ---
  useEffect(() => {
    // This runs once when the App component mounts.
    console.log("Setting up global event listener for engine-tick...");
    
    const unlistenPromise = listen<Record<string, number[]>>('engine-tick', (event) => {
      // Update the store with the new frame data from the backend.
      useFrameStore.setState({ frames: event.payload });
    });

    // Return a cleanup function to remove the listener when the App unmounts.
    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []); // The empty dependency array ensures this runs only once.

  return (
    <main>
      <WledDiscoverer />
    </main>
  );
}

export default App;