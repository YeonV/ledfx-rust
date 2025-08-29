import { useState } from 'react';
import { FileUpload, FileDownload, DeleteForever as DeleteForeverIcon } from "@mui/icons-material";
import { commands } from "@/lib/rust";
import { useStore } from "@/store/useStore";
import { IconBtn } from "@/components/base/IconBtn";
import { ConfirmClearDialog } from './ConfirmClearDialog';
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
        const result = await commands.exportSettings();
        if (result.status === 'ok') {
          engineState = JSON.parse(result.data);
        } else {
          setError(result.error);
          return;
        }
      }

      if (includeFrontend) {

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

          const importedData = JSON.parse(fileContent);


          if (importedData.engine_state) {

            const engineStateString = JSON.stringify(importedData.engine_state);


            const result = await commands.importSettings(engineStateString);

            if (result.status === 'ok') {


              await commands.triggerReload();
            } else {
              setError(result.error);
              return;
            }
          }


          if (importedData.frontend_state) {

            useStore.setState(importedData.frontend_state);
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
    setIsConfirmOpen(false);
    setError(null);
    try {
      const defaultStateResult = await commands.getDefaultEngineState();

      if (defaultStateResult.status === 'ok') {
        const defaultState = defaultStateResult.data;

        const emptyStatePayload = {
          devices: {},
          virtuals: {},
          dsp_settings: defaultState.dsp_settings
        };

        const importResult = await commands.importSettings(JSON.stringify(emptyStatePayload));

        if (importResult.status === 'ok') {
          await commands.triggerReload();
          useStore.setState({ selectedEffects: {}, effectSettings: {} });
        } else {
          setError(importResult.error);
        }
      } else {
        setError(defaultStateResult.error);
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