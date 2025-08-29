import { useEffect, useState } from 'react'
import { Box, Typography } from '@mui/material'
import { commands, DspSettings } from '@/lib/rust'

interface CurveVisualizerProps {
	settings: DspSettings
}

const SVG_WIDTH = 300
const SVG_HEIGHT = 150

const CurveVisualizer = ({ settings }: CurveVisualizerProps) => {
	const [bladeCurve, setBladeCurve] = useState<number[]>([])
	const [customCurve, setCustomCurve] = useState<number[]>([])
	const [error, setError] = useState<string | null>(null)

	// Effect to calculate the reference "Blade" curve once
	useEffect(() => {
		const fetchBladeCurve = async () => {
			const result = await commands.calculateCenterFrequencies(
				settings.num_bands,
				settings.min_freq,
				settings.max_freq,
				'Blade'
			)
			if (result.status === 'ok') {
				setBladeCurve(result.data)
			} else {
				setError(result.error)
			}
		}
		fetchBladeCurve()
	}, [settings.num_bands, settings.min_freq, settings.max_freq]) // Recalculate if range or bands change

	// Effect to calculate the custom curve in real-time as params change
	useEffect(() => {
		if (
			settings.filterbank_type &&
			typeof settings.filterbank_type === 'object' &&
			'BladePlus' in settings.filterbank_type
		) {
			const params = settings.filterbank_type.BladePlus
			const fetchCustomCurve = async () => {
				const result = await commands.calculateCenterFrequencies(
					settings.num_bands,
					settings.min_freq,
					settings.max_freq,
					{ BladePlus: params }
				)
				if (result.status === 'ok') {
					setCustomCurve(result.data)
				} else {
					setError(result.error)
				}
			}
			fetchCustomCurve()
		}
	}, [settings.filterbank_type, settings.num_bands, settings.min_freq, settings.max_freq])

	// --- SVG Path Generation ---
	const generatePath = (curve: number[], maxFreq: number): string => {
		if (curve.length === 0) return ''

		// Convert frequency to a logarithmic X position
		const freqToX = (freq: number) => {
			// Clamp to avoid log(0)
			const safeMin = Math.max(settings.min_freq, 1)
			const logMin = Math.log10(safeMin)
			const logMax = Math.log10(maxFreq)
			const logFreq = Math.log10(Math.max(freq, 1))
			return ((logFreq - logMin) / (logMax - logMin)) * SVG_WIDTH
		}

		// Convert band index to Y position
		const bandToY = (index: number) => SVG_HEIGHT - (index / (curve.length - 1)) * SVG_HEIGHT

		return curve
			.map((freq, index) => {
				const x = freqToX(freq)
				const y = bandToY(index)
				return `${index === 0 ? 'M' : 'L'} ${x.toFixed(2)} ${y.toFixed(2)}`
			})
			.join(' ')
	}

	const bladePath = generatePath(bladeCurve, settings.max_freq)
	const customPath = generatePath(customCurve, settings.max_freq)

	if (error) {
		return <Typography color="error">Error calculating curve: {error}</Typography>
	}

	return (
		<Box
			sx={{
				mb: 2,
				p: 1,
				border: '1px solid',
				borderColor: 'divider',
				borderRadius: 1,
				backgroundColor: 'rgba(0,0,0,0.2)'
			}}
		>
			<svg width="100%" height={SVG_HEIGHT} viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`}>
				{/* Background Grid */}
				<line
					x1={0}
					y1={SVG_HEIGHT / 2}
					x2={SVG_WIDTH}
					y2={SVG_HEIGHT / 2}
					stroke="#444"
					strokeWidth="0.5"
					strokeDasharray="4"
				/>
				<line
					x1={SVG_WIDTH / 2}
					y1={0}
					x2={SVG_WIDTH / 2}
					y2={SVG_HEIGHT}
					stroke="#444"
					strokeWidth="0.5"
					strokeDasharray="4"
				/>

				{/* Reference "Blade" Curve */}
				{bladePath && <path d={bladePath} stroke="rgba(255, 100, 0, 0.4)" strokeWidth="2" fill="none" />}

				{/* Live Custom Curve */}
				{customPath && <path d={customPath} stroke="#4fc3f7" strokeWidth="2.5" fill="none" />}
			</svg>
			{/* TODO: Could add frequency labels to the X-axis here if desired */}
		</Box>
	)
}

export default CurveVisualizer
