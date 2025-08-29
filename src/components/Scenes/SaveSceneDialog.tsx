import { useState, useEffect } from 'react';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle, TextField, Box, Typography, List, ListItem, ListItemText, Checkbox, FormControl, InputLabel, Select, MenuItem, Divider, ListSubheader } from '@mui/material';
import { useStore } from '@/store/useStore';
import { commands, Scene, SceneEffect, EffectConfig } from '@/lib/rust';
import { deepEqual } from '@/utils/deepEqual';

interface SaveSceneDialogProps {
  open: boolean;
  onClose: () => void;
}

export const SaveSceneDialog = ({ open, onClose }: SaveSceneDialogProps) => {
    const { activeEffects, selectedEffects, effectSettings, presetCache, virtuals } = useStore();
    const [sceneName, setSceneName] = useState('My New Scene');
    const [includedVirtuals, setIncludedVirtuals] = useState<Record<string, boolean>>({});
    const [virtualSceneEffects, setVirtualSceneEffects] = useState<Record<string, SceneEffect>>({});

    useEffect(() => {
        if (open) {
            const initialIncludes: Record<string, boolean> = {};
            const initialEffects: Record<string, SceneEffect> = {};

            for (const virtualId in activeEffects) {
                if (!activeEffects[virtualId]) continue;
                initialIncludes[virtualId] = true;

                const effectId = selectedEffects[virtualId];
                const settings = effectSettings[virtualId]?.[effectId];
                if (!effectId || !settings) continue;

                const presets = presetCache[effectId];
                const allPresets = { ...(presets?.user || {}), ...(presets?.built_in || {}) };
                
                let match: string | null = null;
                for (const presetName in allPresets) {
                    if (deepEqual(settings, allPresets[presetName]?.config)) {
                        match = presetName;
                        break;
                    }
                }
                if (match) {
                    initialEffects[virtualId] = { type: "preset", data: { effect_id: effectId, preset_name: match }};
                } else {
                    const payload = { type: effectId as any, config: settings } as EffectConfig;
                    initialEffects[virtualId] = { type: "custom", data: payload };
                }
            }
            setIncludedVirtuals(initialIncludes);
            setVirtualSceneEffects(initialEffects);
        }
    }, [open, activeEffects, selectedEffects, effectSettings, presetCache, virtuals]);

    const handleSaveScene = async () => {
        if (!sceneName.trim()) return;

        const finalVirtualEffects: Record<string, SceneEffect> = {};
        for (const virtualId in includedVirtuals) {
            if (includedVirtuals[virtualId] && virtualSceneEffects[virtualId]) {
                finalVirtualEffects[virtualId] = virtualSceneEffects[virtualId];
            }
        }

        const scenePayload: Scene = {
            id: `scene_${Date.now()}`,
            name: sceneName.trim(),
            virtual_effects: finalVirtualEffects,
        };
        
        await commands.saveScene(scenePayload);
        onClose();
    };

    const handlePresetSelectionForVirtual = (virtualId: string, presetName: string) => {
        const effectId = selectedEffects[virtualId];
        if (presetName === "CUSTOM") {
            const settings = effectSettings[virtualId]?.[effectId];
            const payload = { type: effectId as any, config: settings } as EffectConfig;
            setVirtualSceneEffects(prev => ({ ...prev, [virtualId]: { type: "custom", data: payload }}));
        } else {
            setVirtualSceneEffects(prev => ({ ...prev, [virtualId]: { type: "preset", data: { effect_id: effectId, preset_name: presetName }}}));
        }
    }

    const getPresetValueForVirtual = (virtualId: string): string => {
        const effect = virtualSceneEffects[virtualId];
        if (effect && effect.type === 'preset') {
            return effect.data.preset_name;
        }
        return "CUSTOM";
    }

    return (
        <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
            <DialogTitle>Save New Scene</DialogTitle>
            <DialogContent>
                <TextField autoFocus margin="dense" label="Scene Name" type="text" fullWidth variant="standard" value={sceneName} onChange={(e) => setSceneName(e.target.value)} sx={{ mb: 3 }} />
                <Typography variant="h6" gutterBottom>Active Effects</Typography>
                <Typography variant="body2" color="text.secondary" sx={{mb: 1}}>Select which effects to include and whether to save them as a preset or as custom settings.</Typography>
                <List dense sx={{ border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
                    {Object.keys(activeEffects).filter(id => activeEffects[id]).map((virtualId, index) => {
                        const virtual = virtuals.find(v => v.id === virtualId);
                        const effectId = selectedEffects[virtualId];
                        const presets = presetCache[effectId];
                        const hasUserPresets = presets && Object.keys(presets.user).length > 0;
                        const hasBuiltInPresets = presets && Object.keys(presets.built_in).length > 0;
                        const isFirst = index === 0;

                        return (
                            <>
                            {!isFirst && <Divider />}
                            <ListItem key={virtualId} sx={{display: 'flex', flexDirection: 'column', alignItems: 'flex-start'}}>
                                <Box sx={{display: 'flex', alignItems: 'center', width: '100%'}}>
                                    <Checkbox edge="start" checked={!!includedVirtuals[virtualId]} onChange={(e) => setIncludedVirtuals(prev => ({...prev, [virtualId]: e.target.checked}))} />
                                    <ListItemText primary={virtual?.name || virtualId} secondary={`Effect: ${effectId}`} />
                                </Box>
                                <Box sx={{pl: 6, width: '100%', pb: 1}}>
                                    <FormControl fullWidth size="small" margin="dense">
                                        <InputLabel>Saved As</InputLabel>
                                        <Select value={getPresetValueForVirtual(virtualId)} label="Saved As" onChange={(e) => handlePresetSelectionForVirtual(virtualId, e.target.value)}>
                                            <MenuItem value="CUSTOM"><em>Custom Settings</em></MenuItem>
                                            {hasUserPresets && <ListSubheader>Your Presets</ListSubheader>}
                                            {hasUserPresets && Object.keys(presets.user).map(name => (<MenuItem key={name} value={name}>{name}</MenuItem>))}
                                            {hasBuiltInPresets && <ListSubheader>Built-in Presets</ListSubheader>}
                                            {hasBuiltInPresets && Object.keys(presets.built_in).map(name => (<MenuItem key={name} value={name}>{name}</MenuItem>))}
                                        </Select>
                                    </FormControl>
                                </Box>
                            </ListItem>
                            </>
                        )
                    })}
                </List>
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
                <Button onClick={handleSaveScene}>Save Scene</Button>
            </DialogActions>
        </Dialog>
    );
};