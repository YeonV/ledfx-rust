import { useState } from 'react';
import { Select, MenuItem, FormControl, InputLabel, Button, Box, TextField, Popover, Typography, IconButton, ListSubheader } from '@mui/material';
import { Save as SaveIcon, Delete as DeleteIcon } from '@mui/icons-material';
import type { PresetCollection } from '../../bindings';

interface PresetManagerProps {
  presets?: PresetCollection;
  onLoad: (settings: any) => void;
  onSave: (presetName: string) => void;
  onDelete: (presetName: string) => void;
}

export const PresetManager = ({ presets, onLoad, onSave, onDelete }: PresetManagerProps) => {
  const [selectedPreset, setSelectedPreset] = useState('');
  const [saveAnchorEl, setSaveAnchorEl] = useState<HTMLButtonElement | null>(null);
  const [newPresetName, setNewPresetName] = useState('');

  const handlePresetSelect = (presetName: string) => {
    setSelectedPreset(presetName);
    const userPreset = presets?.user[presetName];
    if (userPreset) {
      onLoad(userPreset);
      return;
    }
    const builtInPreset = presets?.built_in[presetName];
    if (builtInPreset) {
      onLoad(builtInPreset);
    }
  };

  const handleSaveClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setSaveAnchorEl(event.currentTarget);
  };

  const handleSaveConfirm = () => {
    if (newPresetName.trim()) {
      onSave(newPresetName.trim());
      setNewPresetName('');
      setSaveAnchorEl(null);
    }
  };
  
  const handleDelete = () => {
    if(selectedPreset && presets?.user[selectedPreset]) {
        onDelete(selectedPreset);
        setSelectedPreset(''); // Reset selection after delete
    }
  }

  const hasUserPresets = presets && Object.keys(presets.user).length > 0;
  const hasBuiltInPresets = presets && Object.keys(presets.built_in).length > 0;
  const canDelete = !!(selectedPreset && presets?.user[selectedPreset]);

  return (
    <Box sx={{ display: 'flex', gap: 1, alignItems: 'center', pt: 2 }}>
      <FormControl size="small" fullWidth>
        <InputLabel>Presets</InputLabel>
        <Select
          value={selectedPreset}
          label="Presets"
          onChange={(e) => handlePresetSelect(e.target.value)}
          displayEmpty
        >
          <MenuItem value="" disabled><em>Load a Preset</em></MenuItem>
          
          {hasUserPresets && <ListSubheader>Your Presets</ListSubheader>}
          {hasUserPresets && Object.keys(presets.user).map(name => (
            <MenuItem key={name} value={name}>{name}</MenuItem>
          ))}
          
          {hasBuiltInPresets && <ListSubheader>Built-in Presets</ListSubheader>}
          {hasBuiltInPresets && Object.keys(presets.built_in).map(name => (
            <MenuItem key={name} value={name}>{name}</MenuItem>
          ))}

        </Select>
      </FormControl>

      <IconButton onClick={handleSaveClick} color="primary"><SaveIcon /></IconButton>
      <IconButton onClick={handleDelete} color="error" disabled={!canDelete}><DeleteIcon /></IconButton>

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
          <Button onClick={handleSaveConfirm} variant="contained">Save</Button>
        </Box>
      </Popover>
    </Box>
  );
};