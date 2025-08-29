import { Stack, Typography } from '@mui/material'

export const InfoLine = ({ label, value }: { label: string; value: string }) => (
	<Stack direction={'row'} justifyContent={'space-between'} alignItems={'center'}>
		<Typography variant="body2" color="text.secondary" sx={{ mr: 2 }}>
			{label}:
		</Typography>
		<Typography variant="body2" color="text.primary">
			{value}
		</Typography>
	</Stack>
)
