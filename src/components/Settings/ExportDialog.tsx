import { useState } from 'react'
import {
	Button,
	Dialog,
	DialogActions,
	DialogContent,
	DialogTitle,
	TextField,
	FormGroup,
	FormControlLabel,
	Checkbox,
	Typography
} from '@mui/material'

interface ExportDialogProps {
	open: boolean
	onClose: () => void
	onExport: (filename: string, includeEngine: boolean, includeFrontend: boolean) => void
}

export const ExportDialog = ({ open, onClose, onExport }: ExportDialogProps) => {
	const [filename, setFilename] = useState('ledfx-settings.json')
	const [includeEngine, setIncludeEngine] = useState(true)
	const [includeFrontend, setIncludeFrontend] = useState(true)

	const handleExport = () => {
		if (filename && (includeEngine || includeFrontend)) {
			onExport(filename, includeEngine, includeFrontend)
			onClose()
		}
	}

	return (
		<Dialog open={open} onClose={onClose} fullWidth maxWidth="xs">
			<DialogTitle>Export Settings</DialogTitle>
			<DialogContent>
				<Typography variant="body2" gutterBottom>
					Select which parts of your configuration to export.
				</Typography>
				<FormGroup sx={{ my: 2 }}>
					<FormControlLabel
						control={<Checkbox checked={includeEngine} onChange={(e) => setIncludeEngine(e.target.checked)} />}
						label="Engine State (Devices & Virtuals)"
					/>
					<FormControlLabel
						control={<Checkbox checked={includeFrontend} onChange={(e) => setIncludeFrontend(e.target.checked)} />}
						label="UI State (Effect Selections & Settings)"
					/>
				</FormGroup>
				<TextField
					margin="dense"
					label="Filename"
					type="text"
					fullWidth
					variant="standard"
					value={filename}
					onChange={(e) => setFilename(e.target.value)}
				/>
			</DialogContent>
			<DialogActions>
				<Button onClick={onClose}>Cancel</Button>
				<Button onClick={handleExport} variant="contained">
					Export
				</Button>
			</DialogActions>
		</Dialog>
	)
}
