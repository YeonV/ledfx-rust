// src/store/frameStore.ts

import { create } from 'zustand';

interface FrameState {
  frames: Record<string, number[]>;
}

export const useFrameStore = create<FrameState>(() => ({
  frames: {},
}));