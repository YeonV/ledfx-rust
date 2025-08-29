import { Box } from '@mui/material';
import { PlayArrow, Pause, Movie as SceneIcon } from '@mui/icons-material'; // <-- Import Scene icon
import { useStore } from '../../store/useStore';
import { commands } from '../../bindings';
import { SettingsFab } from '../Settings/SettingsFab';
import { MelbankVisualizerFab } from '../MelbankVisualizer/MelbankVisualizerFab';
import { checkEnvironment, isDev } from '../../utils/environment';
import { useEffect } from 'react';
import { SettingsActions } from '../SettingsActions';
import { IconBtn } from '../IconBtn';
import { ScenesFab } from '../Scenes/ScenesFab'; // <-- Import the new component
import DevTools from '../DevTools';

export const RightActions = () => {
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
      <SettingsActions />
      <ScenesFab />
      <IconBtn icon={playbackState.is_paused ? <PlayArrow /> : <Pause />} text={playbackState.is_paused ? "Play" : "Pause"} onClick={handleTogglePause} />
      <MelbankVisualizerFab />
      <SettingsFab />
    </Box>
  );
};