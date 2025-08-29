import { useCallback, useEffect } from 'react'
import { VirtualCard } from '@/components/Virtuals/VirtualCard/VirtualCard'
import { commands, EffectConfig, EffectInfo, EffectSetting, Virtual } from '@lib/rust'
import { useStore } from '@store/useStore'
import { Grid } from '@mui/material'

const buildConfigPayload = (
	effectId: string,
	settings: Record<string, any>,
	availableEffects: EffectInfo[]
): EffectConfig | null => {
	const effectInfo = availableEffects.find((e) => e.id === effectId)
	if (!effectInfo) return null

	return {
		type: effectId as any,
		config: settings
	} as EffectConfig
}

export function Virtuals() {
	const {
		virtuals,
		activeEffects,
		setActiveEffects,
		selectedEffects,
		setSelectedEffects,
		effectSchemas,
		setEffectSchemas,
		effectSettings,
		setEffectSettings,
		availableEffects,
		setPresetsForEffect
	} = useStore()

	useEffect(() => {
		// Clean startup
	}, [])

	// --- REVERTED TO YOUR WORKING VERSION ---
	const handleEffectSelection = useCallback(
		async (virtual: Virtual, newEffectId: string) => {
			const virtualId = virtual.id
			setSelectedEffects({ ...selectedEffects, [virtualId]: newEffectId })

			let schema = effectSchemas[newEffectId]
			if (!schema) {
				try {
					const result = await commands.getEffectSchema(newEffectId)
					if (result.status === 'ok') {
						schema = result.data
						setEffectSchemas({ ...effectSchemas, [newEffectId]: schema })
					} else {
						return
					}
				} catch (e) {
					return
				}
			}

			const effectAlreadyHasSettings = effectSettings[virtualId]?.[newEffectId]
			if (!effectAlreadyHasSettings && schema) {
				const defaultSettings = Object.fromEntries(schema.map((s: EffectSetting) => [s.id, s.defaultValue]))
				const newSettings = {
					...effectSettings,
					[virtualId]: { ...effectSettings[virtualId], [newEffectId]: defaultSettings }
				}
				setEffectSettings(newSettings)

				if (activeEffects[virtualId]) {
					// Pass defaultSettings to start effect
					const configPayload = buildConfigPayload(newEffectId, defaultSettings, availableEffects)
					if (configPayload) {
						await commands.startEffect(virtual.id, configPayload)
					}
				}
			} else {
				if (activeEffects[virtualId]) {
					// Pass existing settings to start effect
					const existingSettings = effectSettings[virtualId]?.[newEffectId]
					const configPayload = buildConfigPayload(newEffectId, existingSettings, availableEffects)
					if (configPayload) {
						await commands.startEffect(virtual.id, configPayload)
					}
				}
			}
		},
		[
			activeEffects,
			effectSchemas,
			effectSettings,
			selectedEffects,
			availableEffects,
			setEffectSchemas,
			setSelectedEffects,
			setEffectSettings
		]
	)

	// --- REVERTED TO YOUR WORKING VERSION ---
	const handleSettingsChange = useCallback(
		(virtualId: string, id: string, value: any) => {
			const effectId = selectedEffects[virtualId]
			if (!effectId) return

			const newSettingsForEffect = { ...effectSettings[virtualId]?.[effectId], [id]: value }
			const newSettings = {
				...effectSettings,
				[virtualId]: { ...effectSettings[virtualId], [effectId]: newSettingsForEffect }
			}
			setEffectSettings(newSettings)

			if (activeEffects[virtualId]) {
				const configPayload = buildConfigPayload(effectId, newSettingsForEffect, availableEffects)
				if (configPayload) {
					commands.updateEffectSettings(virtualId, configPayload).catch(console.error)
				}
			}
		},
		[activeEffects, effectSettings, selectedEffects, availableEffects, setEffectSettings]
	)

	// --- REVERTED TO YOUR WORKING VERSION (with a small simplification) ---
	const handleStartEffect = useCallback(
		async (virtual: Virtual) => {
			const effectId = selectedEffects[virtual.id]
			const settings = effectSettings[virtual.id]?.[effectId]
			if (!effectId || !settings) return

			const configPayload = buildConfigPayload(effectId, settings, availableEffects)
			if (configPayload) {
				try {
					await commands.startEffect(virtual.id, configPayload)
					setActiveEffects({ ...activeEffects, [virtual.id]: true })
				} catch (err) {
					console.error('Failed to start effect:', err)
				}
			}
		},
		[activeEffects, selectedEffects, effectSettings, availableEffects, setActiveEffects]
	)

	// --- REVERTED TO YOUR WORKING VERSION ---
	const handleStopEffect = useCallback(
		async (virtualId: string) => {
			try {
				await commands.stopEffect(virtualId)
				setActiveEffects({ ...activeEffects, [virtualId]: false })
			} catch (err) {
				console.error('Failed to stop effect:', err)
			}
		},
		[activeEffects, setActiveEffects]
	)

	// --- REVERTED TO YOUR WORKING VERSION ---
	const handlePresetLoad = useCallback(
		(virtualId: string, newSettings: EffectConfig) => {
			const effectId = selectedEffects[virtualId]
			if (!effectId) return

			// Your version of this was almost right, but it needed to access .config
			const settingsObject = newSettings.config

			const newSettingsForEffect = { ...effectSettings[virtualId]?.[effectId], ...settingsObject }
			setEffectSettings({
				...effectSettings,
				[virtualId]: { ...effectSettings[virtualId], [effectId]: newSettingsForEffect }
			})

			if (activeEffects[virtualId]) {
				const configPayload = buildConfigPayload(effectId, newSettingsForEffect, availableEffects)
				if (configPayload) {
					commands.updateEffectSettings(virtualId, configPayload).catch(console.error)
				}
			}
		},
		[selectedEffects, effectSettings, activeEffects, availableEffects, setEffectSettings]
	)

	// --- THIS IS THE ONLY FUNCTION THAT NEEDED A REAL CHANGE ---
	const handlePresetSave = useCallback(
		async (virtualId: string, presetName: string) => {
			const effectId = selectedEffects[virtualId]
			const settings = effectSettings[virtualId]?.[effectId]
			if (!effectId || !settings) return

			// This is the one fix: build the full EffectConfig payload
			const settingsPayload = buildConfigPayload(effectId, settings, availableEffects)

			if (settingsPayload) {
				const result = await commands.savePreset(effectId, presetName, settingsPayload)
				if (result.status === 'ok') {
					setPresetsForEffect(effectId, null as any)
				}
			}
		},
		[selectedEffects, effectSettings, availableEffects, setPresetsForEffect]
	)

	// --- REVERTED TO YOUR WORKING VERSION ---
	const handlePresetDelete = useCallback(
		async (virtualId: string, presetName: string) => {
			const effectId = selectedEffects[virtualId]
			if (!effectId) return

			const result = await commands.deletePreset(effectId, presetName)
			if (result.status === 'ok') {
				setPresetsForEffect(effectId, null as any)
			}
		},
		[selectedEffects, setPresetsForEffect]
	)

	return (
		<Grid container spacing={2} sx={{ p: 2 }}>
			{virtuals.map((virtual) => {
				const effectId = selectedEffects[virtual.id]
				return (
					<Grid key={virtual.id}>
						<VirtualCard
							virtual={virtual}
							isActive={activeEffects[virtual.id] || false}
							selectedEffect={effectId}
							schema={effectSchemas[effectId]}
							settings={effectSettings[virtual.id]?.[effectId]}
							onSettingChange={(id, value) => handleSettingsChange(virtual.id, id, value)}
							onEffectSelect={(v, id) => handleEffectSelection(v, id)}
							onStart={() => handleStartEffect(virtual)}
							onStop={() => handleStopEffect(virtual.id)}
							onPresetLoad={(settings) => handlePresetLoad(virtual.id, settings)}
							onPresetSave={(name) => handlePresetSave(virtual.id, name)}
							onPresetDelete={(name) => handlePresetDelete(virtual.id, name)}
						/>
					</Grid>
				)
			})}
		</Grid>
	)
}
