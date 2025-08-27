import { Box, IconButton } from '@mui/material';
import { PlayArrow, Pause } from '@mui/icons-material';
import { useStore } from '../store/useStore';
import { commands } from '../bindings';
import { SettingsFab } from './Settings/SettingsFab';
import { MelbankVisualizerFab } from './MelbankVisualizer/MelbankVisualizerFab';
import DevTools from './DevTools';
import { checkEnvironment, isDev } from '../utils/environment';
import { useEffect } from 'react';
import { ImportExportButtons } from './ImportExportButtons';
import IconBtn from './IconBtn';

export const GlobalControls = () => {
  const { playbackState } = useStore();

  const handleTogglePause = () => {
    commands.togglePause().catch(console.error);
  };

  useEffect(() => {
    checkEnvironment();
  }, []);

  return (
    <Box>
      
      {isDev() && <DevTools />}
      <ImportExportButtons />
      <IconBtn icon={playbackState.is_paused ? <PlayArrow /> : <Pause />} text={playbackState.is_paused ? "Play" : "Pause"} onClick={handleTogglePause} />
      <MelbankVisualizerFab />
      <SettingsFab />
    </Box>
  );
};