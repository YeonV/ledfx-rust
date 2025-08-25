import { useState } from 'react';
import { Add } from '@mui/icons-material';
import { Button } from '@mui/material';
import { commands, type Virtual, type MatrixCell } from "../bindings";
import { AddVirtualDialog } from './AddVirtualDialog';
import { useStore } from '../store/useStore';

export const AddButton = () => {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const allVirtuals = useStore((state) => state.virtuals);

  const handleAddVirtual = async (name: string, selectedDeviceIps: string[]) => {
    // This is the new dynamic logic
    const matrixRow: MatrixCell[] = [];
    
    for (const ip of selectedDeviceIps) {
      const deviceVirtual = allVirtuals.find(v => v.is_device === ip);
      if (deviceVirtual) {
        // A device-virtual's matrix_data is a 1xN array of its pixels
        const deviceCells = deviceVirtual.matrix_data[0]
          .filter((cell): cell is MatrixCell => cell !== null);
        matrixRow.push(...deviceCells);
      }
    }

    const newVirtual: Virtual = {
      id: `custom_${Date.now()}`,
      name: name,
      is_device: null,
      matrix_data: [matrixRow.map(cell => cell)], // Still wrap in Some in Rust if needed, but TS bindings might not require it
    };

    try {
      await commands.addVirtual(newVirtual);
    } catch (e) {
      console.error("Failed to add virtual:", e);
    }
  };

  return (
    <>
      <Button
        variant="contained"
        startIcon={<Add />}
        onClick={() => setIsDialogOpen(true)}
        sx={{ m: 2 }}
      >
        Add Custom Virtual
      </Button>
      <AddVirtualDialog
        open={isDialogOpen}
        onClose={() => setIsDialogOpen(false)}
        onAdd={handleAddVirtual}
      />
    </>
  )
}