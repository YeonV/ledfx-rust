import { useState, useEffect, useMemo } from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle, TextField, Select, MenuItem, FormControl, InputLabel, IconButton, Box, Typography, Paper, List, ListItem, ListItemText, ListItemSecondaryAction, FormHelperText } from '@mui/material';
import { Add as AddIcon, Delete as DeleteIcon, DragHandle as DragHandleIcon } from '@mui/icons-material';
import { useStore } from '@/store/useStore';
import type { Virtual, MatrixCell } from '@/lib/rust';
import { commands } from '@/lib/rust';

interface Segment {
  id: string;
  deviceId: string;
  start?: number;
  end?: number;
  length?: number;
}

interface EditVirtualDialogProps {
  open: boolean;
  onClose: () => void;
  virtualToEdit?: Virtual | null;
}

export const EditVirtualDialog = ({ open, onClose, virtualToEdit }: EditVirtualDialogProps) => {
  const [name, setName] = useState('My Custom Strip');
  const [segments, setSegments] = useState<Segment[]>([]);
  const [newSegment, setNewSegment] = useState<{ deviceId: string; start: number; end: number; length: number } | null>(null);
  const [overlapError, setOverlapError] = useState<string | null>(null);
  
  // --- START: FETCH PHYSICAL DEVICE LIST ---
  // We now get the list of actual devices to know their properties, like led_count.
  const devices = useStore((state) => state.devices);
  // --- END: FETCH PHYSICAL DEVICE LIST ---

  useEffect(() => {
    if (open && virtualToEdit) {
      setName(virtualToEdit.name);
      const decodedSegments: Segment[] = [];
      if (virtualToEdit.matrix_data && virtualToEdit.matrix_data[0]) {
        let currentSegment: Segment | null = null;
        for (const cell of virtualToEdit.matrix_data[0]) {
          if (cell === null) {
            if (currentSegment?.deviceId === 'GAP') {
              currentSegment.length! += 1;
            } else {
              if (currentSegment) decodedSegments.push(currentSegment);
              currentSegment = { id: `seg_${Date.now()}_${decodedSegments.length}`, deviceId: 'GAP', length: 1 };
            }
          } else {
            if (currentSegment && currentSegment.deviceId === cell.device_id && currentSegment.end === cell.pixel - 1) {
              currentSegment.end = cell.pixel;
            } else {
              if (currentSegment) decodedSegments.push(currentSegment);
              currentSegment = { id: `seg_${Date.now()}_${decodedSegments.length}`, deviceId: cell.device_id, start: cell.pixel, end: cell.pixel };
            }
          }
        }
        if (currentSegment) decodedSegments.push(currentSegment);
      }
      setSegments(decodedSegments);
    } else {
      setName('My Custom Strip');
      setSegments([]);
    }
  }, [open, virtualToEdit]);

  const handleAddSegment = () => {
    if (!newSegment) return;
    setOverlapError(null);

    const segmentToAdd: Segment = newSegment.deviceId === 'GAP'
      ? { id: `seg_${Date.now()}`, deviceId: 'GAP', length: newSegment.length }
      : { id: `seg_${Date.now()}`, deviceId: newSegment.deviceId, start: newSegment.start, end: newSegment.end };

    const usedPixels = new Set<string>();
    for (const seg of segments) {
      if (seg.deviceId !== 'GAP') {
        for (let i = seg.start!; i <= seg.end!; i++) {
          usedPixels.add(`${seg.deviceId}-${i}`);
        }
      }
    }
    if (segmentToAdd.deviceId !== 'GAP') {
      for (let i = segmentToAdd.start!; i <= segmentToAdd.end!; i++) {
        const key = `${segmentToAdd.deviceId}-${i}`;
        if (usedPixels.has(key)) {
          setOverlapError(`Overlap detected: Pixel ${i} on device ${segmentToAdd.deviceId} is already in use.`);
          return;
        }
      }
    }

    setSegments([...segments, segmentToAdd]);
    setNewSegment(null);
  };

  const handleSave = async () => {
    if (name && segments.length > 0) {
      const matrixRow: (MatrixCell | null)[] = [];
      for (const segment of segments) {
        if (segment.deviceId === 'GAP') {
          for (let i = 0; i < segment.length!; i++) {
            matrixRow.push(null);
          }
        } else {
          for (let i = segment.start!; i <= segment.end!; i++) {
            matrixRow.push({ device_id: segment.deviceId, pixel: i });
          }
        }
      }

      const virtualPayload: Virtual = {
        id: virtualToEdit?.id || `custom_${Date.now()}`,
        name: name,
        is_device: null,
        matrix_data: [matrixRow],
      };

      try {
        if (virtualToEdit) {
          await commands.updateVirtual(virtualPayload);
        } else {
          await commands.addVirtual(virtualPayload);
        }
        onClose();
      } catch (e) { console.error("Failed to save virtual:", e); }
    }
  };
  
  // --- START: DEVICE-AWARE LOGIC ---
  const selectedDeviceForNewSegment = useMemo(() => 
    devices.find(d => d.ip_address === newSegment?.deviceId),
    [devices, newSegment?.deviceId]
  );

  const maxPixel = useMemo(() => 
    selectedDeviceForNewSegment ? selectedDeviceForNewSegment.led_count - 1 : 0,
    [selectedDeviceForNewSegment]
  );

  const handleDeviceChange = (deviceId: string) => {
    // When device changes, reset the range to prevent invalid states
    setNewSegment({ deviceId, start: 0, end: 0, length: 10 });
  };

  const handleRangeChange = (field: 'start' | 'end', value: number) => {
    if (!newSegment) return;
    let newStart = newSegment.start;
    let newEnd = newSegment.end;

    if (field === 'start') {
      newStart = Math.max(0, Math.min(value, maxPixel));
      if (newStart > newEnd) newEnd = newStart; // Ensure start is not greater than end
    } else {
      newEnd = Math.max(0, Math.min(value, maxPixel));
      if (newEnd < newStart) newStart = newEnd; // Ensure end is not less than start
    }
    setNewSegment({ ...newSegment, start: newStart, end: newEnd });
  };
  // --- END: DEVICE-AWARE LOGIC ---

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>{virtualToEdit ? 'Edit Virtual Strip' : 'Create New Virtual Strip'}</DialogTitle>
      <DialogContent>
        <TextField autoFocus margin="dense" label="Virtual Name" type="text" fullWidth variant="standard" value={name} onChange={(e) => setName(e.target.value)} sx={{ mb: 3 }} />
        <Typography variant="h6" gutterBottom>Segments</Typography>
        <Paper variant="outlined" sx={{ minHeight: 100, p: 1, mb: 2 }}>
          <List dense>
            {segments.map((seg, index) => {
              const deviceName = seg.deviceId === 'GAP' ? 'Gap' : (devices.find(d => d.ip_address === seg.deviceId)?.name || seg.deviceId);
              const secondaryText = seg.deviceId === 'GAP' ? `${seg.length} pixels` : `Pixels ${seg.start} to ${seg.end}`;
              return (
                <ListItem key={seg.id}>
                  <DragHandleIcon sx={{ mr: 1, cursor: 'grab' }} />
                  <ListItemText primary={`${index + 1}. ${deviceName}`} secondary={secondaryText} />
                  <ListItemSecondaryAction>
                    <IconButton edge="end" onClick={() => setSegments(segments.filter(s => s.id !== seg.id))}>
                      <DeleteIcon />
                    </IconButton>
                  </ListItemSecondaryAction>
                </ListItem>
              );
            })}
            {segments.length === 0 && <Typography variant="body2" color="text.secondary" align="center">No segments added yet.</Typography>}
          </List>
        </Paper>

        {newSegment ? (
          <Box>
            <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
              <FormControl fullWidth size="small">
                <InputLabel>Type</InputLabel>
                <Select
                  value={newSegment.deviceId}
                  label="Type"
                  onChange={(e) => handleDeviceChange(e.target.value)}
                >
                  <MenuItem value="GAP">Gap</MenuItem>
                  {devices.map(d => <MenuItem key={d.ip_address} value={d.ip_address}>{d.name}</MenuItem>)}
                </Select>
              </FormControl>
              {newSegment.deviceId === 'GAP' ? (
                <TextField label="Length" type="number" size="small" value={newSegment.length} onChange={e => setNewSegment({...newSegment, length: parseInt(e.target.value) || 0})} />
              ) : (
                <>
                  <TextField label="Start" type="number" size="small" value={newSegment.start} onChange={e => handleRangeChange('start', parseInt(e.target.value) || 0)} inputProps={{ min: 0, max: maxPixel }} />
                  <TextField label="End" type="number" size="small" value={newSegment.end} onChange={e => handleRangeChange('end', parseInt(e.target.value) || 0)} inputProps={{ min: 0, max: maxPixel }} />
                </>
              )}
              <Button onClick={handleAddSegment} variant="contained" size="large">Add</Button>
            </Box>
            {overlapError && <FormHelperText error sx={{mt: 1}}>{overlapError}</FormHelperText>}
          </Box>
        ) : (
          <Button startIcon={<AddIcon />} onClick={() => setNewSegment({ deviceId: devices[0]?.ip_address || 'GAP', start: 0, end: 0, length: 10 })}>
            Add Segment
          </Button>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button onClick={handleSave} variant="contained" disabled={segments.length === 0}>Save</Button>
      </DialogActions>
    </Dialog>
  );
};