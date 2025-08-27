import { useState } from 'react';
import { Button, Stack } from "@mui/material";
import { FileUpload, FileDownload, DeleteForever as DeleteForeverIcon } from "@mui/icons-material";
import { commands } from "../bindings";
import { useStore } from "../store/useStore";
import { IconBtn } from "./IconBtn";
import { ConfirmClearDialog } from './ConfirmClearDialog'; // Import the new dialog
import { ExportDialog } from './ExportDialog';

export const SettingsActions = () => {
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [isExportOpen, setIsExportOpen] = useState(false);
  const setError = useStore((state) => state.setError);

  const handleExport = async (filename: string, includeEngine: boolean, includeFrontend: boolean) => {
    setError(null);
    try {
      let engineState = {};
      let frontendState = {};

      if (includeEngine) {
        const result = await commands.exportSettings(); // This gets the full engine state
        if (result.status === 'ok') {
          engineState = JSON.parse(result.data);
        } else {
          setError(result.error);
          return;
        }
      }

      if (includeFrontend) {
        // Zustand's persist middleware stores the partial state as a stringified object.
        const persistedStateString = localStorage.getItem('ledfx-store');
        if (persistedStateString) {
          frontendState = JSON.parse(persistedStateString).state;
        }
      }
      
      const finalExportObject = {
        ...(includeEngine && { engine_state: engineState }),
        ...(includeFrontend && { frontend_state: frontendState }),
      };

      const jsonString = JSON.stringify(finalExportObject, null, 2);

      const handle = await (window as any).showSaveFilePicker({
        suggestedName: filename,
        types: [{ description: 'JSON Files', accept: { 'application/json': ['.json'] } }],
      });
      const writable = await handle.createWritable();
      await writable.write(jsonString);
      await writable.close();

    } catch (e) {
      if (e instanceof Error && e.name === 'AbortError') {
        console.log('User cancelled the save dialog.');
      } else {
        setError(e as string);
        console.error(e);
      }
    }
  };
  const handleImport = () => {
    setError(null);
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = async (event) => {
        try {
          const fileContent = event.target?.result as string;
          
          // 1. Send the imported file content to the Rust backend.
          const result = await commands.importSettings(fileContent);
          
          if (result.status === 'ok') {
            // 2. Tell the engine to reload its state from the file we just overwrote.
            // The `virtuals-changed` and `devices-changed` events will handle the UI update.
            await commands.triggerReload();
          } else {
            setError(result.error);
          }
        } catch (err) {
          setError(err as string);
          console.error(err);
        }
      };
      reader.readAsText(file);
    };
    
    input.click();
  };
 const handleConfirmClear = async () => {
    setIsConfirmOpen(false); // Close the dialog first
    setError(null);
    try {
      const emptyState = JSON.stringify({ devices: {}, virtuals: {} });
      const result = await commands.importSettings(emptyState);

      if (result.status === 'ok') {
        await commands.triggerReload();
      } else {
        setError(result.error);
      }
    } catch (e) {
      setError(e as string);
    }
  };

  return (
    <>
      <IconBtn icon={<FileUpload />} text="Import" onClick={handleImport} />
      <IconBtn icon={<FileDownload />} text="Export" onClick={() => setIsExportOpen(true)} />
      <IconBtn
        icon={<DeleteForeverIcon />}
        text="Clear All Settings"
        onClick={() => setIsConfirmOpen(true)}
      />
      <ConfirmClearDialog
        open={isConfirmOpen}
        onClose={() => setIsConfirmOpen(false)}
        onConfirm={handleConfirmClear}
      />
      <ExportDialog
        open={isExportOpen}
        onClose={() => setIsExportOpen(false)}
        onExport={handleExport}
      />
    </>
  );
};