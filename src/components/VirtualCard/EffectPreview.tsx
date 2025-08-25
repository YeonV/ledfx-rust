import { useEffect, useRef } from 'react';
import { Box } from '@mui/material';
import { useFrameStore } from '../../store/frameStore';
import { commands } from '../../bindings';

interface EffectPreviewProps {
  virtualId: string; // Changed from ipAddress
  active: boolean;
}

export function EffectPreview({ virtualId, active }: EffectPreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const frame = useFrameStore((state) => state.frames[virtualId]);

  useEffect(() => {
    // The engine now sends preview frames for virtuals, but DDP packets to IPs.
    // Subscriptions are still per-device for network efficiency.
    // We need a way to map virtualId back to the device IPs it uses.
    // For now, we will assume a simple mapping for device-virtuals.
    // TODO: A more robust solution will be needed for multi-device virtuals.
    const ipAddress = virtualId.startsWith('device_') ? virtualId.replace('device_', '') : null;

    if (ipAddress) {
      commands.subscribeToFrames(ipAddress);
    }
    
    return () => {
      if (ipAddress) {
        commands.unsubscribeFromFrames(ipAddress);
      }
    };
  }, [virtualId]);

  useEffect(() => {
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

    if (active) {
      drawFrame(frame);
    } else {
      drawFrame([]); // Clear canvas if not active
    }

  }, [active, frame]);

  return (
    <Box sx={{ border: '1px solid rgba(255, 255, 255, 0.2)', borderRadius: 1, overflow: 'hidden' }}>
      <canvas
        ref={canvasRef}
        style={{ width: '100%', height: '20px', display: 'block' }}
      />
    </Box>
  );
}