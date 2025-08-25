import { create } from 'zustand';
import type { Virtual, AudioDevice, EffectSetting, EffectInfo, Device } from '../bindings';

type EffectSettingsByVirtual = Record<string, Record<string, Record<string, any>>>;

interface IStore {
  devices: Device[];
  setDevices: (devices: Device[]) => void;
  virtuals: Virtual[];
  setVirtuals: (virtuals: Virtual[]) => void;
  isScanning: boolean;
  setIsScanning: (isScanning: boolean) => void;
  error: string | null;
  setError: (error: string | null) => void;
  duration: number;
  setDuration: (duration: number) => void;
  activeEffects: Record<string, boolean>;
  setActiveEffects: (activeEffects: Record<string, boolean>) => void;
  selectedEffects: Record<string, string>;
  setSelectedEffects: (selectedEffects: Record<string, string>) => void;
  targetFps: number;
  setTargetFps: (targetFps: number) => void;
  audioDevices: AudioDevice[];
  setAudioDevices: (audioDevices: AudioDevice[]) => void;
  selectedAudioDevice: string;
  setSelectedAudioDevice: (selectedAudioDevice: string) => void;
  effectSchemas: Record<string, EffectSetting[]>;
  setEffectSchemas: (effectSchemas: Record<string, EffectSetting[]>) => void;
  effectSettings: EffectSettingsByVirtual;
  setEffectSettings: (effectSettings: EffectSettingsByVirtual) => void;
  openSettings: boolean;
  setOpenSettings: (openSettings: boolean) => void;
  openMelbankVisualizer: boolean;
  setOpenMelbankVisualizer: (openMelbankVisualizer: boolean) => void;
  availableEffects: EffectInfo[];
  setAvailableEffects: (effects: EffectInfo[]) => void;
}

export const useStore = create<IStore>((set) => ({
  devices: [],
  setDevices: (devices) => set({ devices }),
  virtuals: [],
  setVirtuals: (virtuals) => set({ virtuals }),
  isScanning: false,
  setIsScanning: (isScanning) => set({ isScanning }),
  error: null,
  setError: (error) => set({ error }),
  duration: 5,
  setDuration: (duration) => set({ duration }),
  activeEffects: {},
  setActiveEffects: (activeEffects) => set({ activeEffects }),
  selectedEffects: {},
  setSelectedEffects: (selectedEffects) => set({ selectedEffects }),
  targetFps: 60,
  setTargetFps: (targetFps) => set({ targetFps }),
  audioDevices: [],
  setAudioDevices: (audioDevices) => set({ audioDevices }),
  selectedAudioDevice: "",
  setSelectedAudioDevice: (selectedAudioDevice) => set({ selectedAudioDevice }),
  effectSchemas: {},
  setEffectSchemas: (effectSchemas) => set({ effectSchemas }),
  effectSettings: {},
  setEffectSettings: (effectSettings) => set({ effectSettings }),
  openSettings: false,
  setOpenSettings: (openSettings) => set({ openSettings }),
  openMelbankVisualizer: false,
  setOpenMelbankVisualizer: (openMelbankVisualizer) => set({ openMelbankVisualizer }),
  availableEffects: [],
  setAvailableEffects: (effects) => set({ availableEffects: effects }),
}));