// src/components/controls/ScanControls.tsx

import { Stack, TextField } from '@mui/material';
import { LoadingButton } from '@mui/lab';
import SearchIcon from '@mui/icons-material/Search';

interface ScanControlsProps {
  duration: number;
  onDurationChange: (duration: number) => void;
  onDiscover: () => void;
  isScanning: boolean;
}

export function ScanControls({ duration, onDurationChange, onDiscover, isScanning }: ScanControlsProps) {
  return (
    <Stack spacing={2} direction="row" alignItems="center" sx={{ mb: 2 }}>
      <TextField
        label="Scan Duration (s)"
        type="number"
        value={duration}
        onChange={(e) => onDurationChange(Number(e.target.value))}
        disabled={isScanning}
        size="small"
      />
      <LoadingButton
        onClick={onDiscover}
        loading={isScanning}
        loadingPosition="start"
        startIcon={<SearchIcon />}
        variant="contained"
      >
        {isScanning ? 'Scanning...' : 'Discover'}
      </LoadingButton>
    </Stack>
  );
}