import { Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle } from '@mui/material';

interface ConfirmClearDialogProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

export const ConfirmClearDialog = ({ open, onClose, onConfirm }: ConfirmClearDialogProps) => {
  return (
    <Dialog
      open={open}
      onClose={onClose}
    >
      <DialogTitle>Clear All Engine Settings?</DialogTitle>
      <DialogContent>
        <DialogContentText>
          Are you sure you want to delete ALL discovered devices and custom virtuals?
          This action cannot be undone.
        </DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button onClick={onConfirm} color="error" variant="contained">
          Clear Settings
        </Button>
      </DialogActions>
    </Dialog>
  );
};