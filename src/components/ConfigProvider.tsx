import { useState, useCallback, type ReactNode } from 'react';
import { Box, Button, Dialog, DialogActions, DialogContent, DialogTitle, Typography } from '@mui/material';
import { commands } from '../bindings';
import { useStore } from '../store/useStore';

// Helper to identify the type of config file
const getConfigType = (data: any): string => {
  if (data.engine_state && data.frontend_state) return 'Full Configuration';
  if (data.devices && data.virtuals) return 'Engine Settings';
  if (data.selectedEffects && data.effectSettings) return 'UI Settings';
  return 'Unknown JSON File';
};

export const ConfigProvider = ({ children }: { children: ReactNode }) => {
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [pendingConfig, setPendingConfig] = useState<{ type: string; data: string } | null>(null);
  const { setError } = useStore();

  const handleFile = useCallback((file: File) => {
    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const content = event.target?.result as string;
        const data = JSON.parse(content);
        const type = getConfigType(data);
        if (type !== 'Unknown JSON File') {
          setPendingConfig({ type, data: content });
          setIsConfirmOpen(true);
        } else {
          setError('Unrecognized JSON file format');
        }
      } catch (error) {
        setError('Failed to parse JSON file');
      }
    };
    reader.readAsText(file);
  }, [setError]);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer.files[0];
    if (file && file.type === 'application/json') {
      handleFile(file);
    }
  }, [handleFile]);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  const handleConfirmImport = async () => {
    if (!pendingConfig) return;
    try {
      const result = await commands.importSettings(pendingConfig.data);
      if (result.status === 'ok') {
        // We must reload the app for the backend and frontend to pick up all changes
        window.location.reload();
      } else {
        setError(result.error);
      }
    } catch (e) {
      setError(e as string);
    }
    setIsConfirmOpen(false);
    setPendingConfig(null);
  };

  return (
    <Box sx={{ height: '100vh', width: '100vw' }} onDrop={handleDrop} onDragOver={handleDragOver}>
      {children}
      <Dialog open={isConfirmOpen} onClose={() => setIsConfirmOpen(false)}>
        <DialogTitle>Import Configuration</DialogTitle>
        <DialogContent>
          <Typography>
            Detected a "{pendingConfig?.type}" file.
          </Typography>
          <Typography>
            Would you like to import it? This will overwrite your current settings and reload the application.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsConfirmOpen(false)}>Cancel</Button>
          <Button onClick={handleConfirmImport} variant="contained">Import & Reload</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};