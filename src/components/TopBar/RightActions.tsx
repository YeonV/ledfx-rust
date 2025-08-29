import { Box } from '@mui/material'
import { PlayArrow, Pause } from '@mui/icons-material'
import { useStore } from '@/store/useStore'
import { commands } from '@/lib/rust'
import { SettingsFab } from '../Settings/SettingsFab'
import { MelbankVisualizerFab } from '../MelbankVisualizer/MelbankVisualizerFab'
import { checkEnvironment, isDev } from '@/utils/environment'
import { useEffect } from 'react'
import { SettingsActions } from '../Settings/SettingsActions'
import { IconBtn } from '../base/IconBtn'
import { ScenesFab } from '../Scenes/ScenesFab'
import DevTools from '../DevTools/DevTools'

export const RightActions = () => {
	const { playbackState } = useStore()

	const handleTogglePause = () => {
		commands.togglePause().catch(console.error)
	}

	useEffect(() => {
		checkEnvironment()
	}, [])

	return (
		<Box>
			{isDev() && <DevTools />}
			<SettingsActions />
			<ScenesFab />
			<IconBtn
				icon={playbackState.is_paused ? <PlayArrow /> : <Pause />}
				text={playbackState.is_paused ? 'Play' : 'Pause'}
				onClick={handleTogglePause}
			/>
			<MelbankVisualizerFab />
			<SettingsFab />
		</Box>
	)
}
