// src/components/EffectPreview.tsx

import { useEffect, useRef } from 'react';
import { Box } from '@mui/material';
import { useFrameStore } from '../store/frameStore';
import { invoke } from '@tauri-apps/api/core';

interface EffectPreviewProps {
  ipAddress: string;
  active: boolean;
}

export function EffectPreview({ ipAddress, active }: EffectPreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    // When the component mounts, tell the backend we are interested in this IP.
    invoke('subscribe_to_frames', { ipAddress });
    console.log(`Subscribed to frames for ${ipAddress}`);

    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const drawFrame = (pixelData: number[] | undefined) => {
      if (!ctx || !canvas) return;
      const p = pixelData || [];
      const numLeds = p.length / 3;

      const { width, height } = canvas.getBoundingClientRect();
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }

      if (numLeds === 0) {
        ctx.fillStyle = 'black';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        return;
      }
      
      const ledWidth = canvas.width / numLeds;
      for (let i = 0; i < numLeds; i++) {
        const r = p[i * 3];
        const g = p[i * 3 + 1];
        const b = p[i * 3 + 2];
        ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
        ctx.fillRect(i * ledWidth, 0, Math.ceil(ledWidth), canvas.height);
      }
    };

    if (!active) {
      drawFrame([]);
    }

    const unsubscribeFromStore = useFrameStore.subscribe(
      (state) => {
        if (active) {
          const pixels = state.frames[ipAddress];
          drawFrame(pixels);
        }
      }
    );

    // The cleanup function
    return () => {
      unsubscribeFromStore();
      // Tell the backend we are no longer interested in this IP.
      invoke('unsubscribe_from_frames', { ipAddress });
      console.log(`Unsubscribed from frames for ${ipAddress}`);
    };
  }, [active, ipAddress]);

  return (
    <Box sx={{ border: '1px solid rgba(255, 255, 255, 0.2)', borderRadius: 1, overflow: 'hidden' }}>
      <canvas
        ref={canvasRef}
        style={{ width: '100%', height: '20px', display: 'block' }}
      />
    </Box>
  );
}