import { useEffect, useState } from 'react';
import { commands } from '../bindings';

const MelbankVisualizer = () => {
  const [melbanks, setMelbanks] = useState<number[]>([]);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const data = await commands.getAudioAnalysis();
        console.log(data)
        setMelbanks(data.melbanks);
      } catch (error) {
        console.error('Failed to get audio analysis:', error);
      }
    }, 1000 / 30); // Fetch data at 30 FPS

    return () => clearInterval(interval);
  }, []);

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'flex-end',
        width: '100%',
        height: '100px',
        border: '1px solid #444',
        backgroundColor: '#111',
        padding: '2px',
        boxSizing: 'border-box',
        gap: '1px',
      }}
    >
      {melbanks?.map((value, index) => {
        // Clamp and scale the value for display
        const height = Math.min(Math.max(value * 100, 0), 100);
        return (
          <div
            key={index}
            style={{
              width: '100%',
              height: `${height}%`,
              backgroundColor: `hsl(${(index * 360) / melbanks.length}, 100%, 50%)`,
            }}
          />
        );
      })}
    </div>
  );
};

export default MelbankVisualizer;