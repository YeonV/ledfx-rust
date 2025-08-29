import { useEffect, useRef, useState } from 'react';
import { commands } from '@/lib/rust';
const MelbankVisualizer = () => {
    const [melbanks, setMelbanks] = useState<number[]>([]);
    const canvasRef = useRef<HTMLCanvasElement>(null);
    // This effect handles the data fetching loop
    useEffect(() => {
        const interval = setInterval(async () => {
            try {
                const result = await commands.getAudioAnalysis();
                if (result.status === "ok") {
                    setMelbanks(result.data.melbanks);
                }
            } catch (error) {
                // It's common for this to fail during hot-reloads, so we won't log errors
            }
        }, 1000 / 60); // Fetch data at 60 FPS for smoothness
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas || !melbanks || melbanks.length === 0) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const { width, height } = canvas;
        ctx.clearRect(0, 0, width, height);
        ctx.fillStyle = '#111';
        ctx.fillRect(0, 0, width, height);

        const barWidth = width / melbanks.length;

        for (let i = 0; i < melbanks.length; i++) {
            const value = melbanks[i];
            const barHeight = Math.min(Math.max(value * height, 0), height);

            const hue = (i * 360) / melbanks.length;
            ctx.fillStyle = `hsl(${hue}, 100%, 50%)`;

            ctx.fillRect(i * barWidth, height - barHeight, barWidth, barHeight);
        }
    }, [melbanks]);
    
    return (
        <canvas
            ref={canvasRef}
            width="500" // Set explicit canvas dimensions
            height="100"
            style={{
                width: '100%',
                height: '100px',
                border: '1px solid #444',
                boxSizing: 'border-box',
            }}
        />
    );
};
export default MelbankVisualizer;