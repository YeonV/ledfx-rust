import { Box, IconButton } from '@mui/material';
import { PlayArrow, Pause } from '@mui/icons-material';
import { useStore } from '../store/useStore';
import { commands } from '../bindings';
import { SettingsFab } from './Settings/SettingsFab';
import { MelbankVisualizerFab } from './MelbankVisualizer/MelbankVisualizerFab';
import { WledDiscoverer } from './WledDiscoverer';

export const GlobalControls = () => {
  const { playbackState, devices } = useStore();

  const handleTogglePause = () => {
    commands.togglePause().catch(console.error);
  };

  return (
    <Box>
      <IconButton onClick={handleTogglePause}>
        {playbackState.is_paused ? <PlayArrow /> : <Pause />}
      </IconButton>
      <MelbankVisualizerFab />
      <SettingsFab />
    </Box>
  );
};