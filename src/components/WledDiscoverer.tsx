// src/components/WledDiscoverer.tsx

import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "../bindings";
import { useStore } from "../store/useStore";
import { Wled } from "./Icons/Icons";
import { Box, LinearProgress, Button, Alert } from "@mui/material";
import type { WledDevice } from "../bindings";

export function WledDiscoverer() {
  const {
    devices, setDevices,
    isScanning, setIsScanning,
    error, setError,
    duration,
    setAudioDevices,
    setSelectedAudioDevice,
  } = useStore();

  useEffect(() => {
    const setupAudio = async () => {
      const result = await commands.getAudioDevices();
      if (result.status === "ok") {
        setAudioDevices(result.data);
        if (result.data.length > 0) {
          const defaultDevice = result.data[0].name;
          setSelectedAudioDevice(defaultDevice);
          await commands.setAudioDevice(defaultDevice);
        }
      } else {
        setError(result.error);
      }
    };

    setupAudio().catch(console.error);

    const unlistenPromise = listen<WledDevice>("wled-device-found", (event) => {
      const foundDevice = event.payload;
      if (!devices.some((d) => d.ip_address === foundDevice.ip_address)) {
        setDevices([...devices, foundDevice]);
      }
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);


  const handleDiscover = async () => {
    setIsScanning(true);
    setError(null);
    setDevices([]);
    try {
      await commands.discoverWled(duration);
      setTimeout(() => setIsScanning(false), duration * 1000);
    } catch (err) {
      setError(err as string);
      setIsScanning(false);
    }
  };

  return (
    <Box sx={{ width: "100%", p: 2 }}>
      <Button
        onClick={handleDiscover}
        loading={isScanning}
        loadingPosition="start"
        startIcon={<Wled />}
        variant="contained"
      >
        {isScanning ? "Scanning..." : "Discover"}
      </Button>

      {isScanning && <LinearProgress sx={{ mb: 2 }} />}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}      
    </Box>
  );
}
