import { Alert, Box } from '@mui/material'
import { useStore } from '@/store/useStore'
import { useEvents } from '@/hooks/useEvents'
import { useInit } from '@/hooks/useInit'
import { ConfigDrop } from '@/components/Settings/ConfigDrop'
import { Virtuals } from '@/components/Virtuals/Virtuals'
import { TopBar } from '@/components/TopBar/TopBar'
import './App.css'

function App() {
	const { virtuals, error } = useStore()

	useInit()
	useEvents()

	return (
		<Box>
			<ConfigDrop />
			<TopBar />
			{error && (
				<Alert severity="error" sx={{ mt: 2, mb: 2 }}>
					{error}
				</Alert>
			)}
			<main style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 48px)', overflowY: 'auto' }}>
				{virtuals.length > 0 && <Virtuals />}
			</main>
		</Box>
	)
}

export default App
