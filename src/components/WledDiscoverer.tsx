import { useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "../bindings";
import { useStore } from "../store/useStore";
import { Wled } from "./Icons/Icons";
import { IconBtn } from "./IconBtn";
import type { WledDevice, Device } from "../bindings";

export function WledDiscoverer() {
  const {
    isScanning, setIsScanning,
    setError,
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
        setError(result.error as string);
      }
    };

    setupAudio().catch(console.error);

    const unlistenPromise = listen<WledDevice>("wled-device-found", (event) => {
      const foundDevice = event.payload;
      const devicePayload: Device = {
        ip_address: foundDevice.ip_address,
        name: foundDevice.name,
        led_count: foundDevice.leds.count,
      };
      commands.addDevice(devicePayload).catch(console.error);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [setAudioDevices, setSelectedAudioDevice, setError]);

  const handleDiscover = useCallback(async () => {
    setIsScanning(true);
    setError(null);
    
    try {
      await commands.discoverWled(duration);
      setTimeout(() => setIsScanning(false), duration * 1000);
    } catch (err) {
      setError(err as string);
      setIsScanning(false);
    }
  }, [duration, setIsScanning, setError]);

  return (
    <IconBtn text="Discover WLED" onClick={handleDiscover} disabled={isScanning} icon={<Wled width={20} scan={isScanning} />} />
  );
}