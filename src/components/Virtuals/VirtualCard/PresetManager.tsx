import { useState, useEffect, useMemo } from 'react'
import {
	Select,
	MenuItem,
	FormControl,
	InputLabel,
	Button,
	Box,
	TextField,
	Popover,
	Typography,
	IconButton,
	ListSubheader
} from '@mui/material'
import { Save as SaveIcon, Delete as DeleteIcon } from '@mui/icons-material'
import type { PresetCollection, EffectConfig } from '@/lib/rust'
import { deepEqual } from '@/utils/deepEqual'

interface PresetManagerProps {
	presets?: PresetCollection
	settings: Record<string, any> // Current settings for the effect
	onLoad: (settings: EffectConfig) => void
	onSave: (presetName: string) => void
	onDelete: (presetName: string) => void
}

export const PresetManager = ({ presets, settings, onLoad, onSave, onDelete }: PresetManagerProps) => {
	// We no longer manage `selectedPresetName` directly; it will be `currentMatch`
	const [saveAnchorEl, setSaveAnchorEl] = useState<HTMLButtonElement | null>(null)
	const [newPresetName, setNewPresetName] = useState('')

	const allPresets = useMemo(() => ({ ...(presets?.user || {}), ...(presets?.built_in || {}) }), [presets])

	// --- START: NEW AUTO-MATCHING LOGIC ---
	const [currentMatch, setCurrentMatch] = useState<string | null>(null)

	useEffect(() => {
		let matchedPreset: string | null = null
		for (const presetName in allPresets) {
			const presetConfig = allPresets[presetName] // This is an EffectConfig
			if (!presetConfig || !presetConfig.config) continue

			// console.log(settings, presetConfig.config, deepEqual(settings, presetConfig.config))
			// Compare the current settings with the preset's INNER config
			if (deepEqual(settings, presetConfig.config)) {
				matchedPreset = presetName
				break
			}
		}
		setCurrentMatch(matchedPreset)
	}, [settings, allPresets])

	// `isDirty` is now true if no preset matches the current settings
	const isDirty = useMemo(() => currentMatch === null, [currentMatch])
	// --- END: NEW AUTO-MATCHING LOGIC ---

	const handlePresetSelect = (presetName: string) => {
		const presetToLoad = allPresets[presetName]
		if (presetToLoad) {
			onLoad(presetToLoad)
			// The useEffect will auto-set currentMatch based on the loaded settings
		}
	}

	const handleSaveClick = (event: React.MouseEvent<HTMLButtonElement>) => {
		setSaveAnchorEl(event.currentTarget)
		// Pre-fill the name with currentMatch if it exists, otherwise empty
		setNewPresetName(currentMatch || '')
	}

	const handleSaveConfirm = () => {
		if (newPresetName.trim()) {
			onSave(newPresetName.trim())
			setNewPresetName('')
			setSaveAnchorEl(null)
		}
	}

	const handleDelete = () => {
		if (currentMatch && presets?.user[currentMatch]) {
			// Only delete user presets
			onDelete(currentMatch)
			setCurrentMatch(null) // Deselect after delete
		}
	}

	const hasUserPresets = presets && Object.keys(presets.user).length > 0
	const hasBuiltInPresets = presets && Object.keys(presets.built_in).length > 0
	// Can only delete if a preset is selected AND it's a user preset
	const canDelete = !!(currentMatch && presets?.user[currentMatch])

	return (
		<Box sx={{ display: 'flex', gap: 1, alignItems: 'center', pt: 2 }}>
			<FormControl size="small" fullWidth>
				<InputLabel shrink>Presets</InputLabel>
				<Select
					value={currentMatch || ''} // Always reflect the currently matched preset
					label="Presets"
					onChange={(e) => handlePresetSelect(e.target.value)}
					displayEmpty
					//   renderValue={(selected) => {
					//     if (!selected) {
					//         return <em>Custom Settings</em>
					//     }
					//     return selected;
					//   }}
				>
					<MenuItem value="" disabled>
						<em>Custom Settings</em>
					</MenuItem>

					{hasUserPresets && <ListSubheader>Your Presets</ListSubheader>}
					{hasUserPresets &&
						Object.keys(presets.user).map((name) => (
							<MenuItem key={name} value={name}>
								{name}
							</MenuItem>
						))}

					{hasBuiltInPresets && <ListSubheader>Built-in Presets</ListSubheader>}
					{hasBuiltInPresets &&
						Object.keys(presets.built_in).map((name) => (
							<MenuItem key={name} value={name}>
								{name}
							</MenuItem>
						))}
				</Select>
			</FormControl>

			{/* Save button is disabled if `!isDirty` */}
			<IconButton onClick={handleSaveClick} color="primary" disabled={!isDirty}>
				<SaveIcon />
			</IconButton>
			<IconButton onClick={handleDelete} color="primary" disabled={!canDelete}>
				<DeleteIcon />
			</IconButton>

			<Popover
				open={Boolean(saveAnchorEl)}
				anchorEl={saveAnchorEl}
				onClose={() => setSaveAnchorEl(null)}
				anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
				transformOrigin={{ vertical: 'top', horizontal: 'right' }}
			>
				<Box sx={{ p: 2, display: 'flex', flexDirection: 'column', gap: 1 }}>
					<Typography variant="subtitle2">Save Current Settings</Typography>
					<TextField
						autoFocus
						label="Preset Name"
						size="small"
						value={newPresetName}
						onChange={(e) => setNewPresetName(e.target.value)}
						onKeyDown={(e) => e.key === 'Enter' && handleSaveConfirm()}
					/>
					<Button onClick={handleSaveConfirm} variant="contained">
						Save
					</Button>
				</Box>
			</Popover>
		</Box>
	)
}
