import { useEffect } from 'react'
import { commands, DspSettings as DspSettingsType } from '../../bindings'
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
	TextField, // <-- Import TextField
	Alert,
	Box // <-- Import Alert
} from '@mui/material'
import TrackChangesIcon from '@mui/icons-material/TrackChanges'
import SpeedIcon from '@mui/icons-material/Speed'
import SettingsIcon from '@mui/icons-material/Settings'
import VolumeUpIcon from '@mui/icons-material/VolumeUp'
import { useStore } from '../../store/useStore'
import DspSettings from './DspSettings'
import { ArrowDropDown, Close, SettingsSuggest, Language } from '@mui/icons-material' // <-- Import Language icon
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
		apiPort,
		setApiPort
	} = useStore()

	const handleAudioDeviceChange = (event: SelectChangeEvent<string>) => {
		const deviceName = event.target.value
		setSelectedAudioDevice(deviceName)
		commands.setAudioDevice(deviceName).catch(console.error)
	}

	useEffect(() => {
		const handler = setTimeout(() => {
			commands.setTargetFps(targetFps).catch(console.error)
		}, 500)
		return () => clearTimeout(handler)
	}, [targetFps])

	// --- START: NEW HANDLER FOR API PORT ---
	const handleApiPortChange = (port: number) => {
		// Update the backend. This saves the setting to disk.
		commands.setApiPort(port).catch(console.error)
		// Update our local state so the UI reflects the change immediately.
		setApiPort(port)
	}
	// --- END: NEW HANDLER FOR API PORT ---

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
				<Select fullWidth value={selectedAudioDevice} onChange={handleAudioDeviceChange}>
					{audioDevices.map((device) => (
						<MenuItem key={device.name} value={device.name}>
							{device.name}
						</MenuItem>
					))}
				</Select>
			</SettingsRow>

			{/* --- START: NEW API SETTINGS SECTION --- */}
			<Accordion elevation={0} defaultExpanded>
				<AccordionSummary expandIcon={<ArrowDropDown />}>
					<Language sx={{ mr: 2 }} />
					<Typography variant="h6">API Server</Typography>
				</AccordionSummary>
				<AccordionDetails sx={{ p: 0 }}>
					<SettingsRow title="Server Port">
						<TextField
							type="number"
							size="small"
							value={apiPort || 3030}
							onChange={(e) => handleApiPortChange(parseInt(e.target.value, 10) || 3030)}
							inputProps={{ min: 1024, max: 65535 }}
						/>
					</SettingsRow>
					<Box sx={{ px: 2, pb: 2 }}>
						<Alert severity="info">A restart of the application is required for port changes to take effect.</Alert>
					</Box>
				</AccordionDetails>
			</Accordion>
			{/* --- END: NEW API SETTINGS SECTION --- */}

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
