// src/components/controls/EngineControls.tsx

import { Box, Slider, Typography } from '@mui/material';
import { commands } from '../../bindings';
import { useEffect, useState } from 'react';

export function EngineControls() {
  const [targetFps, setTargetFps] = useState(60);

  useEffect(() => {
    const handler = setTimeout(() => {
      commands.setTargetFps(targetFps).catch(console.error);
    }, 500);
    return () => clearTimeout(handler);
  }, [targetFps]);

  return (
    <Box sx={{ width: 300, mb: 2 }}>
      <Typography gutterBottom>Target FPS: {targetFps}</Typography>
      <Slider
        value={targetFps}
        onChange={(_e, newValue) => setTargetFps(newValue as number)}
        aria-labelledby="target-fps-slider"
        valueLabelDisplay="auto"
        step={5}
        marks
        min={10}
        max={120}
      />
    </Box>
  );
}