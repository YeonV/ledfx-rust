import { useState, useEffect } from 'react';
import { Box, Popover, Typography, List, ListItem, ListItemText, IconButton, Divider, Button, TextField } from '@mui/material';
import { Collections as SceneIcon, PlayArrow as PlayArrowIcon, Delete as DeleteIcon, Add as AddIcon } from '@mui/icons-material';
import { commands } from '../../bindings';
import { useStore } from '../../store/useStore';
import { IconBtn } from '../base/IconBtn';
import { SaveSceneDialog } from './SaveSceneDialog';

export const ScenesFab = () => {
    const scenes = useStore((state) => state.scenes);
    const activeSceneId = useStore((state) => state.activeSceneId);
    const [sceneManagerAnchor, setSceneManagerAnchor] = useState<HTMLElement | null>(null);
    const [isSaveDialogOpen, setIsSaveDialogOpen] = useState(false);

    const handleOpenSceneManager = (event: React.MouseEvent<HTMLElement>) => {
    setSceneManagerAnchor(event.currentTarget);
    };

    const handleCloseSceneManager = () => {
    setSceneManagerAnchor(null);
    };


    const handleActivateScene = (sceneId: string) => {
        commands.activateScene(sceneId).catch(console.error);
        handleCloseSceneManager(); // Close the manager after activating a scene
    };

    const handleDeleteScene = (sceneId: string) => {
        // Here we would ideally pop a confirmation dialog
        commands.deleteScene(sceneId).catch(console.error);
    };

    return (<>
    
      <IconBtn icon={<SceneIcon />} text="Scenes" onClick={handleOpenSceneManager} />
        <Popover
            open={Boolean(sceneManagerAnchor)}
            anchorEl={sceneManagerAnchor}
            onClose={handleCloseSceneManager}
            anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
            transformOrigin={{ vertical: 'top', horizontal: 'right' }}
            slotProps={{ paper: { sx: { width: 300 }}}}
        >
            <Box sx={{ p: 2, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <Typography variant="h6" component="div">
                    Scenes
                </Typography>
                <Button 
                    startIcon={<AddIcon />} 
                    variant="contained" 
                    size="small"
                    onClick={() => {
                        setIsSaveDialogOpen(true);
                        handleCloseSceneManager(); // Close the popover when opening the dialog
                    }}
                >
                    Save New
                </Button>
            </Box>
            <Divider />
            <List dense>
                {scenes.length === 0 && (
                    <ListItem>
                        <ListItemText primary="No scenes saved yet." />
                    </ListItem>
                )}
                {scenes.map((scene) => (
                    <ListItem
                        key={scene.id}
                        sx={{ bgcolor: scene.id === activeSceneId ? 'action.selected' : 'transparent' }}
                        secondaryAction={
                            <>
                                <IconButton edge="end" aria-label="activate" onClick={() => handleActivateScene(scene.id)}>
                                    <PlayArrowIcon />
                                </IconButton>
                                <IconButton edge="end" aria-label="delete" onClick={() => handleDeleteScene(scene.id)}>
                                    <DeleteIcon />
                                </IconButton>
                            </>
                        }
                    >
                        <ListItemText primary={scene.name} />
                    </ListItem>
                ))}
            </List>
        </Popover>
        <SaveSceneDialog
            open={isSaveDialogOpen}
            onClose={() => setIsSaveDialogOpen(false)}
        />
    </>
    );
};