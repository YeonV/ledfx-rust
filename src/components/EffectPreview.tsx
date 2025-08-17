// src/components/EffectPreview.tsx

import { useEffect, useRef } from 'react';
import { Box } from '@mui/material';
import { listen } from '@tauri-apps/api/event';

interface EffectFramePayload {
  ip_address: string;
  pixels: number[];
}

interface EffectPreviewProps {
  ipAddress: string;
  active: boolean;
}

export function EffectPreview({ ipAddress, active }: EffectPreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  // Use a ref to store the latest pixel data without causing re-renders.
  const lastPixelsRef = useRef<number[]>([]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // A reusable function to draw a frame of pixels to the canvas.
    const drawFrame = (pixels: number[]) => {
      if (!ctx || !canvas) return;
      const numLeds = pixels.length / 3;
      // If there are no LEDs, draw a black bar and exit.
      if (numLeds === 0) {
        ctx.fillStyle = 'black';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        return;
      }
      
      const ledWidth = canvas.width / numLeds;

      for (let i = 0; i < numLeds; i++) {
        const r = pixels[i * 3];
        const g = pixels[i * 3 + 1];
        const b = pixels[i * 3 + 2];
        ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
        // Use Math.ceil to prevent tiny gaps between LEDs on fractional widths.
        ctx.fillRect(i * ledWidth, 0, Math.ceil(ledWidth), canvas.height);
      }
    };

    // If the effect becomes inactive, clear the last pixels and draw black.
    if (!active) {
      lastPixelsRef.current = [];
      drawFrame([]);
    }

    // --- THE FIX: Use a ResizeObserver to sync canvas resolution ---
    const observer = new ResizeObserver(entries => {
      // Get the actual size of the canvas element on the page.
      const { width, height } = entries[0].contentRect;
      // Update the canvas's internal drawing resolution.
      canvas.width = width;
      canvas.height = height;
      // Redraw the last known frame to the newly sized canvas.
      drawFrame(lastPixelsRef.current);
    });
    observer.observe(canvas);

    // Listen for frame updates from the backend.
    const unlistenPromise = listen<EffectFramePayload>('effect-frame-update', (event) => {
      if (event.payload.ip_address === ipAddress) {
        // Store the latest frame and draw it.
        lastPixelsRef.current = event.payload.pixels;
        drawFrame(event.payload.pixels);
      }
    });

    // Cleanup function to disconnect the observer and remove the listener.
    return () => {
      observer.disconnect();
      unlistenPromise.then(unlisten => unlisten());
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