import { IconButton, IconButtonProps, Tooltip } from '@mui/material'

export type IconBtnProps = {
	icon: React.ReactNode
	text: string
} & IconButtonProps

export const IconBtn = ({ icon, text, ...props }: IconBtnProps) => {
	return (
		<Tooltip title={text}>
			<IconButton {...props}>{icon}</IconButton>
		</Tooltip>
	)
}
