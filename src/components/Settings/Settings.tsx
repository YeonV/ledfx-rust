import { useEffect, useState } from 'react'
import { commands } from '../../bindings'
import {
	Stack,
	Slider,
	Typography,
	Card,
	Select,
	MenuItem,
	SelectChangeEvent,
	IconButton,
	Accordion,
	AccordionSummary,
	AccordionDetails,
	TextField,
	Box,
	Button
} from '@mui/material'
import TrackChangesIcon from '@mui/icons-material/TrackChanges'
import SpeedIcon from '@mui/icons-material/Speed'
import SettingsIcon from '@mui/icons-material/Settings'
import VolumeUpIcon from '@mui/icons-material/VolumeUp'
import { useStore } from '../../store/useStore'
import DspSettings from './DspSettings'
import {
	ArrowDropDown,
	Close,
	SettingsSuggest,
	Language,
	Save as SaveIcon,
	RestartAlt as RestartAltIcon
} from '@mui/icons-material'
import { SettingsRow } from './SettingsRow'

export function Settings() {
	const {
		duration,
		setDuration,
		targetFps,
		setTargetFps,
		audioDevices,
		selectedAudioDevice,
		setSelectedAudioDevice,
		setOpenSettings,
		apiPort: savedApiPort,
		setApiPort
	} = useStore()

	const [localApiPort, setLocalApiPort] = useState(savedApiPort)
	const [defaultApiPort, setDefaultApiPort] = useState(savedApiPort || 3030) // Store the default

	// Fetch the default port when the component mounts
	useEffect(() => {
		commands.getDefaultEngineState().then((result) => {
			if (result.status === 'ok' && result.data.api_port) {
				setDefaultApiPort(result.data.api_port)
			}
		})
	}, [])

	useEffect(() => {
		setLocalApiPort(savedApiPort)
	}, [savedApiPort])

	const handleAudioDeviceChange = (event: SelectChangeEvent<string>) => {
		setSelectedAudioDevice(event.target.value)
		commands.setAudioDevice(event.target.value).catch(console.error)
	}

	useEffect(() => {
		const handler = setTimeout(() => {
			commands.setTargetFps(targetFps).catch(console.error)
		}, 500)
		return () => clearTimeout(handler)
	}, [targetFps])

	const handleSaveApiPort = () => {
		commands
			.setApiPort(localApiPort)
			.then((result) => {
				if (result.status === 'ok') {
					setApiPort(localApiPort)
				}
			})
			.catch(console.error)
	}

	// --- START: NEW RESET HANDLER ---
	const handleResetApiPort = () => {
		setLocalApiPort(defaultApiPort)
	}
	// --- END: NEW RESET HANDLER ---

	const isPortDirty = localApiPort !== savedApiPort

	return (
		<Stack spacing={0}>
			<Card variant="outlined">
				<Stack direction="row" alignItems="center" justifyContent={'space-between'} p={2}>
					<Stack direction="row" alignItems="center" spacing={2}>
						<SettingsIcon />
						<Typography variant="h6">Settings</Typography>
					</Stack>
					<IconButton onClick={() => setOpenSettings(false)}>
						<Close />
					</IconButton>
				</Stack>
			</Card>

			<SettingsRow icon={<TrackChangesIcon />} title={`Scan Duration: ${duration}s`}>
				<Slider
					value={duration}
					onChange={(_e, newValue) => setDuration(Number(newValue))}
					min={1}
					max={30}
					step={1}
					marks
					valueLabelDisplay="auto"
				/>
			</SettingsRow>
			<SettingsRow icon={<SpeedIcon />} title={`Target: ${targetFps} FPS`}>
				<Slider
					value={targetFps}
					onChange={(_e, newValue) => setTargetFps(newValue as number)}
					min={10}
					max={120}
					step={5}
					marks
					valueLabelDisplay="auto"
				/>
			</SettingsRow>
			<SettingsRow icon={<VolumeUpIcon />} title={'Audio Input'}>
				<Select
					disableUnderline
					size="small"
					variant="standard"
					label="Audio Device"
					value={selectedAudioDevice}
					onChange={handleAudioDeviceChange}
				>
				{audioDevices.map((device) => {
					const cleanName = device.name.replace('System Audio ', '')
					const parts = cleanName.split(' (')
					const mainName = parts[0].replace('(', '')
					const metadata = parts[1] ? '(' + parts[1].replace('))', ')') : ''
					
					return (
						<MenuItem key={device.name} value={device.name} sx={{ justifyContent: 'space-between', display: 'flex' }}>
							<Typography variant="body2" pr={2} display={'inline-flex'}>
								{device.name.startsWith('System Audio') ? '🔊' : '🎤'} {mainName}
							</Typography>
							{metadata && (
								<Typography variant="caption" color="text.secondary" display={'inline-flex'}>
									{metadata}
								</Typography>
							)}
						</MenuItem>
					)
				})}
				</Select>
			</SettingsRow>

			<Accordion elevation={0} defaultExpanded>
				<AccordionSummary expandIcon={<ArrowDropDown />}>
					<Language sx={{ mr: 2 }} />
					<Typography variant="h6">API Server</Typography>
				</AccordionSummary>
				<AccordionDetails sx={{ p: 2, pt: 0 }}>
					<SettingsRow title="Server Port">
						<Stack direction="row" alignItems="center">
							<IconButton onClick={handleSaveApiPort} disabled={!isPortDirty} size="small" sx={{ mr: 1 }}>
								<SaveIcon />
							</IconButton>
							<IconButton onClick={handleResetApiPort} disabled={!isPortDirty} size="small" sx={{ mr: 1 }}>
								<RestartAltIcon />
							</IconButton>
							<TextField
								type="number"
								size="small"
								value={localApiPort}
								onChange={(e) => setLocalApiPort(parseInt(e.target.value, 10) || 3030)}
								inputProps={{ min: 1024, max: 65535 }}
							/>
						</Stack>
					</SettingsRow>
				</AccordionDetails>
			</Accordion>

			<Accordion elevation={0}>
				<AccordionSummary expandIcon={<ArrowDropDown />}>
					<SettingsSuggest sx={{ mr: 2 }} />
					<Typography variant="h6">Advanced DSP Settings</Typography>
				</AccordionSummary>
				<AccordionDetails sx={{ p: 0 }}>
					<DspSettings />
				</AccordionDetails>
			</Accordion>
		</Stack>
	)
}
