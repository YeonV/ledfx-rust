import { useEffect } from 'react';
import { useStore } from '../store/useStore';
import { commands } from '../bindings';

export const useInit = () => {
    const { 
        setAvailableEffects, 
        setVirtuals, 
        setDevices, 
        setDspSettings, 
        setScenes, 
        setAudioDevices, 
        setSelectedAudioDevice 
    } = useStore();

    useEffect(() => {
        const fetchInitialState = async () => {
          try {
            // A helper to run commands and handle errors
            const run = async <T,>(cmd: Promise<{ status: "ok"; data: T } | { status: "error"; error: any }>, setter: (data: T) => void) => {
                const result = await cmd;
                if (result.status === 'ok') {
                    setter(result.data);
                } else {
                    console.error("Failed to fetch initial state:", result.error);
                }
            }

            // Fetch everything in parallel for faster startup
            await Promise.all([
                run(commands.getAvailableEffects(), setAvailableEffects),
                run(commands.getVirtuals(), setVirtuals),
                run(commands.getDevices(), setDevices),
                run(commands.getScenes(), setScenes),
                run(commands.getDspSettings(), setDspSettings),
                (async () => {
                    const audioResult = await commands.getAudioDevices();
                    if (audioResult.status === "ok") {
                        const { devices, default_device_name } = audioResult.data;
                        setAudioDevices(devices);
            
                        const storedDevice = useStore.getState().selectedAudioDevice;
                        const deviceToSelect = storedDevice || default_device_name || devices[0]?.name;
            
                        if (deviceToSelect) {
                            setSelectedAudioDevice(deviceToSelect);
                            await commands.setAudioDevice(deviceToSelect);
                        }
                    } else {
                        console.error("Failed to get audio devices:", audioResult.error);
                    }
                })()
            ]);

          } catch (e) { console.error("Failed to fetch initial state:", e); }
        };

        fetchInitialState();
    // The dependency array ensures this runs only once on mount
    }, [setAvailableEffects, setVirtuals, setDevices, setDspSettings, setScenes, setAudioDevices, setSelectedAudioDevice]);
};