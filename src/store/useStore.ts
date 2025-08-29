import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Virtual, AudioDevice, EffectSetting, EffectInfo, Device, PlaybackState, DspSettings, PresetCollection, Scene } from '../lib/rust';

type EffectSettingsByVirtual = Record<string, Record<string, Record<string, any>>>;

type PresetCache = Record<string, PresetCollection>;

type IStore = {
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
  playbackState: PlaybackState;
  setPlaybackState: (state: PlaybackState) => void;
  dspSettings: DspSettings | null;
  setDspSettings: (settings: DspSettings) => void;
  dirtyDspSettings: DspSettings | null;
  setDirtyDspSettings: (settings: DspSettings) => void;
  presetCache: PresetCache;
  setPresetsForEffect: (effectId: string, presets: PresetCollection) => void;
  scenes: Scene[];
  setScenes: (scenes: Scene[]) => void;
  activeSceneId: string | null;
  setActiveSceneId: (id: string | null) => void;
}

export const useStore = create<IStore>()(
  persist(
    (set) => ({
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
      playbackState: { is_paused: false },
      setPlaybackState: (state) => set({ playbackState: state }),
      dspSettings: null,
      setDspSettings: (settings) => set({ dspSettings: settings, dirtyDspSettings: settings }),
      dirtyDspSettings: null,
      setDirtyDspSettings: (settings) => set({ dirtyDspSettings: settings }),
      presetCache: {},
      setPresetsForEffect: (effectId, presets) => set((state) => ({
        presetCache: { ...state.presetCache, [effectId]: presets }
      })),
      scenes: [],
      setScenes: (scenes) => set({ scenes }),
      activeSceneId: null,
      setActiveSceneId: (id) => set({ activeSceneId: id }),
    }),
    {
      name: 'ledfx-store',
      partialize: (state) =>
        Object.fromEntries(
          Object.entries(state).filter(([key]) => [
            'selectedAudioDevice',
            'dirtyDspSettings',
            // Scenes are not persisted in the frontend store; they are fetched from the backend.
          ].includes(key))
        ),
    },
  )
);