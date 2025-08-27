import { Button, IconButton, Stack } from "@mui/material";
import { FileUpload, FileDownload } from "@mui/icons-material";
import { commands } from "../bindings";
import { useStore } from "../store/useStore";
import IconBtn from "./IconBtn";

export const ImportExportButtons = () => {
  const setError = useStore((state) => state.setError);

  // const handleExport = async () => {
  //   setError(null);
  //   try {
  //     // 1. Get the engine state string from the Rust backend.
  //     const result = await commands.exportSettings();
  //     if (result.status === 'ok') {
  //       const jsonString = result.data;

  //       const blob = new Blob([jsonString], { type: 'application/json' });
  //       const url = URL.createObjectURL(blob);
        
  //       // 2. Use the "Web Way" to trigger a download of the backend state.
  //       const a = document.createElement('a');
  //       a.href = url;
  //       a.download = 'ledfx-engine-settings.json';
  //       document.body.appendChild(a);
  //       a.click();
  //       document.body.removeChild(a);       
        
  //       URL.revokeObjectURL(url);
  //       console.log("Exported settings:", url);
  //     } else {
  //       setError(result.error);
  //     }
  //   } catch (e) {
  //     setError(e as string);
  //     console.error(e);
  //   }
  // };

  const handleExport = async () => {
  setError(null);
  try {
    const result = await commands.exportSettings();
    if (result.status === 'ok') {
      const jsonString = result.data;

      // The new, modern way
      const handle = await (window as any).showSaveFilePicker({
        suggestedName: 'ledfx-engine-settings.json',
        types: [{
          description: 'JSON Files',
          accept: { 'application/json': ['.json'] },
        }],
      });

      // Create a writable stream to the file handle
      const writable = await handle.createWritable();
      // Write our JSON string to the file
      await writable.write(jsonString);
      // Close the file and write the contents to disk
      await writable.close();

    } else {
      setError(result.error);
    }
  } catch (e) {
    // This will catch errors if the user cancels the save dialog
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

  return (
    <>
      <IconBtn icon={<FileUpload />} text="Import" onClick={handleImport} />
      <IconBtn icon={<FileDownload />} text="Export" onClick={handleExport} />
    </>
  );
};