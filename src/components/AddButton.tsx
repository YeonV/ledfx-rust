import { useState } from 'react';
import { Add } from '@mui/icons-material';
import { IconButton } from '@mui/material';
import { EditVirtualDialog } from './EditVirtualDialog';

export function AddButton() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  return (
    <>
      <IconButton onClick={() => setIsDialogOpen(true)}>
        <Add />
      </IconButton>
      <EditVirtualDialog
        open={isDialogOpen}
        onClose={() => setIsDialogOpen(false)}
      />
    </>
  )
}