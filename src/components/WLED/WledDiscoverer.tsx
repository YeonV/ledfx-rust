import { useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "../../store/useStore";
import { Wled } from "../base/Icons/Icons";
import { IconBtn } from "../base/IconBtn";
import { type WledDevice, type Device, commands } from "../../lib/rust";

export function WledDiscoverer() {
  const { isScanning, setIsScanning, setError, duration } = useStore();

  const handleDiscover = useCallback(async () => {
    setIsScanning(true);
    setError(null);

    const unlisten = await listen<WledDevice>("wled-device-found", (event) => {
      const foundDevice = event.payload;
      const devicePayload: Device = {
        ip_address: foundDevice.ip_address,
        name: foundDevice.name,
        led_count: foundDevice.leds.count,
      };
      commands.addDevice(devicePayload).catch(console.error);
    });
    
    try {
      await commands.discoverWled(duration);
      setTimeout(() => {
        setIsScanning(false);
        unlisten();
      }, duration * 1000);
    } catch (err) {
      setError(err as string);
      setIsScanning(false);
      unlisten();
    }
  }, [duration, setIsScanning, setError]);

  return (
    <IconBtn text="Discover WLED" onClick={handleDiscover} disabled={isScanning} icon={<Wled width={20} scan={isScanning} />} />
  );
}