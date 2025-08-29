import { useState, useCallback, useEffect } from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle, Typography } from '@mui/material';
import { commands } from '@/lib/rust';
import { useStore } from '@/store/useStore';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { readTextFile } from '@tauri-apps/plugin-fs';

// Helper to identify the type of config file
const getConfigType = (data: any): string => {
  if (data.engine_state && data.frontend_state) return 'Full Configuration';
  if (data.devices && data.virtuals) return 'Engine Settings';
  if (data.selectedEffects && data.effectSettings) return 'UI Settings';
  return 'Unknown JSON File';
};

export const ConfigDrop = () => {
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [pendingConfig, setPendingConfig] = useState<{ type: string; data: string } | null>(null);
  const { setError } = useStore();

  const handleFileContent = useCallback((content: string) => {
    try {
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
  }, [setError]);

  useEffect(() => {
    const unlistenPromise = getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type === 'drop') {
        const filePath = event.payload.paths[0]; // Get the first dropped file path
        if (filePath && filePath.toLowerCase().endsWith('.json')) {
          try {
            const content = await readTextFile(filePath);
            handleFileContent(content); 
          } catch (error) {
            console.error('File drop error:', error);
            setError('Failed to read file');
          }
        }
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [handleFileContent]);

  const handleConfirmImport = async () => {
    if (!pendingConfig) return;
    try {
      const importedData = JSON.parse(pendingConfig.data);

      if (importedData.engine_state) {
        const engineStateString = JSON.stringify(importedData.engine_state);
        const result = await commands.importSettings(engineStateString);
        if (result.status === 'ok') {
          await commands.triggerReload();
        } else {
          setError(result.error);
          setIsConfirmOpen(false);
          setPendingConfig(null);
          return;
        }
      }

      if (importedData.frontend_state) {
        useStore.setState(importedData.frontend_state);
      }
    } catch (e) {
      setError(e as string);
    }
    setIsConfirmOpen(false);
    setPendingConfig(null);
  };

  return (
    <Dialog open={isConfirmOpen} onClose={() => setIsConfirmOpen(false)}>
      <DialogTitle>Import Configuration</DialogTitle>
      <DialogContent>
        <Typography>Detected a "{pendingConfig?.type}" file.</Typography>
        <Typography>Would you like to import it? This will overwrite your current settings.</Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setIsConfirmOpen(false)}>Cancel</Button>
        <Button onClick={handleConfirmImport} variant="contained">Import</Button>
      </DialogActions>
    </Dialog>
  );
};