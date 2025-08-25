import { useState } from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle, TextField, Select, MenuItem, FormControl, InputLabel, OutlinedInput, Box, Chip } from '@mui/material';
import { useStore } from '../store/useStore';
import type { Device } from '../bindings';

interface AddVirtualDialogProps {
  open: boolean;
  onClose: () => void;
  onAdd: (name: string, selectedDeviceIps: string[]) => void;
}

export const AddVirtualDialog = ({ open, onClose, onAdd }: AddVirtualDialogProps) => {
  const [name, setName] = useState('My Custom Strip');
  const [selectedDeviceIps, setSelectedDeviceIps] = useState<string[]>([]);
  
  // Get the list of physical devices from the store
  const allVirtuals = useStore((state) => state.virtuals);
  const physicalDevices = allVirtuals.filter(v => v.is_device);

  const handleAdd = () => {
    if (name && selectedDeviceIps.length > 0) {
      onAdd(name, selectedDeviceIps);
      onClose();
      // Reset state for next time
      setName('My Custom Strip');
      setSelectedDeviceIps([]);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="xs">
      <DialogTitle>Add New Virtual Strip</DialogTitle>
      <DialogContent>
        <TextField
          autoFocus
          margin="dense"
          label="Virtual Name"
          type="text"
          fullWidth
          variant="standard"
          value={name}
          onChange={(e) => setName(e.target.value)}
          sx={{ mb: 3 }}
        />
        <FormControl fullWidth>
          <InputLabel>Select Devices (in order)</InputLabel>
          <Select
            multiple
            value={selectedDeviceIps}
            onChange={(e) => setSelectedDeviceIps(e.target.value as string[])}
            input={<OutlinedInput label="Select Devices (in order)" />}
            renderValue={(selected) => (
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                {selected.map((ip) => {
                  const deviceName = physicalDevices.find(d => d.is_device === ip)?.name || ip;
                  return <Chip key={ip} label={deviceName} />;
                })}
              </Box>
            )}
          >
            {physicalDevices.map((virtual) => (
              <MenuItem key={virtual.id} value={virtual.is_device!}>
                {virtual.name}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button onClick={handleAdd} variant="contained">Add</Button>
      </DialogActions>
    </Dialog>
  );
};