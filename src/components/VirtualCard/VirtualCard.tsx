import { memo, useState } from 'react';
import { InfoLine } from './Infoline';
import { EffectPicker } from './EffectPicker';
import { EffectPreview } from './EffectPreview';
import { EffectSettings } from './EffectSettings';
import { Accordion, AccordionDetails, AccordionSummary, Card, CardContent, CardHeader, IconButton, Stack, Tooltip, Typography } from '@mui/material';
import { ArrowDropDown, Lightbulb as LightbulbIcon, PlayArrow as PlayArrowIcon, Stop as StopIcon, Edit as EditIcon} from '@mui/icons-material';
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
}: VirtualCardProps) => {
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
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
        <EffectPicker virtual={virtual} selectedEffect={selectedEffect} onEffectSelect={onEffectSelect} />
        {schema && settings && <Accordion elevation={0} sx={{ mt: 1.5, border: '1px solid #444', borderRadius: '4px', overflow: 'hidden', minHeight: 40 }}>
          <AccordionSummary expandIcon={<ArrowDropDown />} sx={{ pr: 0.8}}>
            <Typography>Effect Settings</Typography>
          </AccordionSummary>
          <AccordionDetails>
            <EffectSettings schema={schema} settings={settings} onSettingChange={onSettingChange} />
          </AccordionDetails>
        </Accordion>}
      </CardContent>
    </Card>
    <EditVirtualDialog
        open={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        virtualToEdit={virtual} // Pass the virtual to open in "Edit" mode
      />
      </>
  );
});