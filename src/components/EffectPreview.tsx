// src/components/EffectPreview.tsx

import { useEffect, useRef } from 'react';
import { Box } from '@mui/material';

interface EffectPreviewProps {
  pixels: number[] | undefined; // Receives pixels as a prop
}

export function EffectPreview({ pixels }: EffectPreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  // Use a ref to store the latest pixel data for the resize observer.
  const pixelsRef = useRef<number[] | undefined>(pixels);
  pixelsRef.current = pixels;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const drawFrame = (pixelData: number[] | undefined) => {
      if (!ctx || !canvas) return;
      const p = pixelData || [];
      const numLeds = p.length / 3;

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

    const observer = new ResizeObserver(entries => {
      const { width, height } = entries[0].contentRect;
      canvas.width = width;
      canvas.height = height;
      // Redraw the most recent frame on resize.
      drawFrame(pixelsRef.current);
    });
    observer.observe(canvas);

    // Draw the initial frame when the component mounts or pixels change.
    drawFrame(pixels);

    return () => {
      observer.disconnect();
    };
  }, [pixels]); // Re-run this effect only when the pixels prop itself changes.

  return (
    <Box sx={{ border: '1px solid rgba(255, 255, 255, 0.2)', borderRadius: 1, overflow: 'hidden' }}>
      <canvas
        ref={canvasRef}
        style={{ width: '100%', height: '20px', display: 'block' }}
      />
    </Box>
  );
}