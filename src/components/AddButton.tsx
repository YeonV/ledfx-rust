import { useState } from 'react';
import { Add } from '@mui/icons-material';
import { Button } from '@mui/material';
import { EditVirtualDialog } from './EditVirtualDialog';

export function AddButton() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);

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
      <EditVirtualDialog
        open={isDialogOpen}
        onClose={() => setIsDialogOpen(false)}
        // We pass no `virtualToEdit` prop, so it opens in "Create" mode
      />
    </>
  )
}