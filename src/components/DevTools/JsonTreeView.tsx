// src/components/JsonTreeView.jsx
import { useMemo, useState } from 'react'
import { Box, Typography, IconButton, Collapse, List, ListItem, ListItemIcon, ListItemText } from '@mui/material'
import { ExpandMore, ChevronRight, Functions as FunctionsIcon } from '@mui/icons-material'

const getDataType = (data: any) => {
	if (data === null) return 'null'
	if (Array.isArray(data)) return 'array'
	if (typeof data === 'function') return 'function'
	return typeof data
}

const Value = ({ value }: { value: any }) => {
	const type = getDataType(value)
	if (type === 'function') {
		return (
			<Box component="span" sx={{ display: 'flex', alignItems: 'center', color: '#666' /* VS Code function color */ }}>
				<FunctionsIcon sx={{ fontSize: '1rem', mr: 0.5 }} />
				<Typography component="span" sx={{ fontStyle: 'italic', fontFamily: 'monospace' }}>
					function
				</Typography>
			</Box>
		)
	}
	let color
	switch (type) {
		case 'string':
			color = '#9ccc65'
			break // Green
		case 'number':
			color = '#4fc3f7'
			break // Blue
		case 'boolean':
			color = '#ff8a65'
			break // Orange
		case 'null':
			color = '#e0e0e0'
			break // Gray
		default:
			color = '#ba68c8'
			break // Purple for undefined
	}
	return (
		<Typography component="span" sx={{ color, fontFamily: 'monospace' }}>
			{JSON.stringify(value)}
		</Typography>
	)
}

function JsonTreeView({
	data,
	nodeKey = 'state',
	defaultOpen = false
}: {
	data: any
	nodeKey?: string
	defaultOpen?: boolean
}) {
	const [isOpen, setIsOpen] = useState(defaultOpen)
	const dataType = getDataType(data)

	const isCollapsible = dataType === 'object' || dataType === 'array'

	const entries = useMemo(() => {
		if (!isCollapsible) return []
		let objectEntries = Object.entries(data)
		if (dataType === 'object') {
			const getTypePriority = (value: any) => {
				const valueType = getDataType(value)
				if (valueType === 'object' || valueType === 'array') {
					return 1
				}
				if (valueType === 'function') {
					return 2
				}
				return 0
			}

			objectEntries.sort(([keyA, valueA], [keyB, valueB]) => {
				const priorityA = getTypePriority(valueA)
				const priorityB = getTypePriority(valueB)
				if (priorityA !== priorityB) {
					return priorityA - priorityB
				}
				return keyA.localeCompare(keyB)
			})
		}
		return objectEntries
	}, [data, isCollapsible, dataType])

	const handleToggle = (e: React.MouseEvent<HTMLLIElement>) => {
		e.stopPropagation()
		setIsOpen(!isOpen)
	}

	if (!isCollapsible) {
		return (
			<ListItem sx={{ pl: 2 }}>
				<ListItemText
					primary={
						<Typography
							component="span"
							sx={{ color: dataType === 'function' ? '#999' : '#ce9178', fontFamily: 'monospace' }}
						>
							"{nodeKey}":{' '}
						</Typography>
					}
					secondary={<Value value={data} />}
					sx={{ m: 0, display: 'flex', alignItems: 'center' }}
				/>
			</ListItem>
		)
	}

	return (
		<List component="div" disablePadding sx={{ width: '100%' }}>
			<ListItem onClick={handleToggle} sx={{ p: '2px 8px' }}>
				<ListItemIcon sx={{ minWidth: 24 }}>
					<IconButton size="small" sx={{ color: 'white' }}>
						{isOpen ? <ExpandMore /> : <ChevronRight />}
					</IconButton>
				</ListItemIcon>
				<ListItemText
					primary={
						<Typography component="span" sx={{ color: '#ce9178', fontFamily: 'monospace' }}>
							"{nodeKey}":{' '}
						</Typography>
					}
					secondary={
						<Typography component="span" sx={{ color: '#d4d4d4', fontFamily: 'monospace' }}>
							{dataType === 'array' ? `Array(${entries.length})` : 'Object'}
						</Typography>
					}
				/>
			</ListItem>
			<Collapse in={isOpen} timeout="auto" unmountOnExit>
				<List component="div" disablePadding sx={{ pl: 2, borderLeft: '1px solid #444' }}>
					{entries.map(([key, value]) => (
						<JsonTreeView key={key} nodeKey={key} data={value} />
					))}
				</List>
			</Collapse>
		</List>
	)
}

export default JsonTreeView
