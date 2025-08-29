import { memo, useState } from 'react';
import { InfoLine } from './Infoline';
import { EffectPicker } from './EffectPicker';
import { EffectPreview } from './EffectPreview';
import { EffectSettings } from './EffectSettings';
import { Box, Button, Card, CardContent, CardHeader, Collapse, IconButton, Stack, Tooltip } from '@mui/material';
import { Lightbulb as LightbulbIcon, PlayArrow as PlayArrowIcon, Stop as StopIcon, Edit as EditIcon, Tune as TuneIcon } from '@mui/icons-material';
import { type EffectSetting, type Virtual } from '../../bindings';
import { EditVirtualDialog } from '../EditVirtualDialog';

interface VirtualCardProps {
  virtual: Virtual;
  isActive: boolean;
  selectedEffect: string;
  onEffectSelect: (virtual: Virtual, effectId: string) => void;
  onStart: (virtual: Virtual) => void;
  onStop: (virtualId: string) => void;
  onSettingChange: (id: string, value: any) => void;
  schema?: EffectSetting[];
  settings?: Record<string, any>;
  
  // --- START: NEW PROPS FOR PRESETS ---
  onPresetLoad: (settings: any) => void;
  onPresetSave: (presetName: string) => void;
  onPresetDelete: (presetName: string) => void;
  // --- END: NEW PROPS FOR PRESETS ---
}

export const VirtualCard = memo(({
  virtual,
  isActive,
  selectedEffect,
  schema,
  settings,
  onEffectSelect,
  onStart,
  onStop,
  onSettingChange,
  // --- START: NEW PROPS FOR PRESETS ---
  onPresetLoad,
  onPresetSave,
  onPresetDelete,
  // --- END: NEW PROPS FOR PRESETS ---
}: VirtualCardProps) => {
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [openEffectSettings, setOpenEffectSettings] = useState(false);
  const pixelCount = virtual.matrix_data?.flat()?.filter(Boolean)?.length || 0;

  return (
    <>
    <Card variant="outlined" sx={{ height: 'auto', width: '350px', display: 'flex', flexDirection: 'column' }}>
      <CardHeader
        sx={{ p: "8px 8px"}}
        avatar={<LightbulbIcon color={isActive ? 'warning' : 'inherit'} />}
        title={<Stack direction="row" alignItems="center">
          {virtual.name}
          <Tooltip sx={{ ml: 2}} title={<>
              <InfoLine label="Type" value={virtual.is_device ? 'Device' : 'Custom'} />
              <InfoLine label="ID" value={virtual.id} />
              <InfoLine label="Pixels" value={`${pixelCount}`} />
              {virtual.is_device && <InfoLine label="IP Address" value={virtual.is_device} />}
          </>
          }>
            <span style={{ color: '#666', border: '1px solid #444', padding: '0px 8px', borderRadius: '4px', fontSize: '12px', marginLeft: 16 }}>{'INFO'}</span>
          </Tooltip></Stack>}          
        action={
          <>
            {!virtual.is_device && (
              <IconButton size="small" onClick={() => { setIsEditDialogOpen(true) }}>
                <EditIcon />
              </IconButton>
            )}
            <IconButton
              onClick={() => isActive ? onStop(virtual.id) : onStart(virtual)}
            >
              {isActive ? <StopIcon /> : <PlayArrowIcon />}
            </IconButton>
          </>
        }
      />
      <CardContent sx={{ p: '8px', pb: '8px !important' }}>
        <EffectPreview virtualId={virtual.id} active={isActive} />
        <Stack direction={'row'} alignItems="center">
          <EffectPicker virtual={virtual} selectedEffect={selectedEffect} onEffectSelect={onEffectSelect} />
          <Button disabled={!settings} variant='outlined' sx={{ mt: '12px', minWidth: 40, p: 0, height: 40, borderColor: '#444', color: 'text.primary' }} size='large' onClick={() => {setOpenEffectSettings(!openEffectSettings)}}>
              <TuneIcon />
          </Button>
        </Stack>
        {schema && settings && <Collapse in={openEffectSettings} sx={{ mt: 0, p:0, overflow: 'hidden', minHeight: 1 }}>
          <Box px={0}>
            <EffectSettings 
              schema={schema} 
              settings={settings} 
              onSettingChange={onSettingChange}
              effectId={selectedEffect}
              onPresetLoad={onPresetLoad}
              onPresetSave={onPresetSave}
              onPresetDelete={onPresetDelete}
            />
          </Box>
        </Collapse>}
      </CardContent>
    </Card>
    <EditVirtualDialog
        open={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        virtualToEdit={virtual}
      />
      </>
  );
});