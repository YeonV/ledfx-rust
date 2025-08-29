// src/components/DevTools.jsx
import { useState, useMemo, useRef, useEffect, MouseEvent } from 'react';
import { Paper, Box, Typography, TextField, InputAdornment, IconButton } from '@mui/material';
import { Search, Clear, DragIndicator, BugReport } from '@mui/icons-material';
import { useStore } from '@/store/useStore';
import { IconBtn } from '../base/IconBtn';
import JsonTreeView from './JsonTreeView';


// A recursive function to filter the state object based on the search term
const filterState = (state: ReturnType<typeof useStore.getState>, term: string): typeof state | undefined => {
  if (!term) return state;
  const lowerCaseTerm = term.toLowerCase();

  const filter = (data: any): any => {
    if (Array.isArray(data)) {
      const filteredArray = data.map((item) => filter(item)).filter(item => item !== undefined);
      return filteredArray.length > 0 ? filteredArray : undefined;
    }
    if (typeof data === 'object' && data !== null) {
      const filteredObj = Object.entries(data).reduce((acc: { [key: string]: any }, [key, value]) => {
        if (key.toLowerCase().includes(lowerCaseTerm)) {
          acc[key] = value; // If key matches, include the whole subtree
          return acc;
        }
        const filteredValue = filter(value);
        if (filteredValue !== undefined) {
          acc[key] = filteredValue;
        }
        return acc;
      }, {} as { [key: string]: any });
      return Object.keys(filteredObj).length > 0 ? filteredObj : undefined;
    }
    // For primitives, check if they match
    if (String(data).toLowerCase().includes(lowerCaseTerm)) {
      return data;
    }
    return undefined;
  };

  return filter(state);
};

function DevTools() {
  const state = useStore();
  const [open, setOpen] = useState<boolean>(false);
  const [searchTerm, setSearchTerm] = useState<string>('');
  const [position, setPosition] = useState<{ x: number; y: number }>({ x: 20, y: 20 });
  const dragRef = useRef<{ isDragging: boolean; offsetX: number; offsetY: number }>({ isDragging: false, offsetX: 0, offsetY: 0 });

  const filteredState = useMemo<typeof state | undefined>(() => filterState(state, searchTerm), [state, searchTerm]);

  const handleMouseDown = (e: MouseEvent<HTMLDivElement>) => {
    dragRef.current = {
      isDragging: true,
      offsetX: e.clientX - position.x,
      offsetY: e.clientY - position.y,
    };
  };

  const handleMouseMove = (e: MouseEvent | globalThis.MouseEvent) => {
    if (!dragRef.current.isDragging) return;
    setPosition({
      x: e.clientX - dragRef.current.offsetX,
      y: e.clientY - dragRef.current.offsetY,
    });
  };

  const handleMouseUp = () => {
    dragRef.current.isDragging = false;
  };

  useEffect(() => {
    window.addEventListener('mousemove', handleMouseMove as EventListener);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove as EventListener);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return (<>
  <IconBtn icon={<BugReport />} text="Open DevTools" onClick={() => setOpen(!open)} />
  {open && <Paper
      elevation={8}
      sx={{
        position: 'fixed',
        left: `${position.x}px`,
        top: `${position.y}px`,
        zIndex: 9999,
        width: 800,
        maxHeight: '70vh',
        backgroundColor: '#1e1e1e',
        color: '#d4d4d4',
        display: 'flex',
        flexDirection: 'column',
        fontFamily: 'monospace',
      }}
    >
      <Box
        onMouseDown={handleMouseDown}
        sx={{
          p: 1,
          backgroundColor: '#333',
          cursor: 'move',
          display: 'flex',
          alignItems: 'center',
          borderTopLeftRadius: '4px',
          borderTopRightRadius: '4px',
        }}
      >
        <DragIndicator sx={{ mr: 1, color: '#888' }} />
        <Typography variant="h6" sx={{ fontSize: '1rem', flexGrow: 1 }}>Zustand State</Typography>
      </Box>

      <Box sx={{ p: 1 }}>
        <TextField
          fullWidth
          variant="outlined"
          size="small"
          placeholder="Search state..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          sx={{
            '& .MuiOutlinedInput-root': {
              color: 'white',
              '& fieldset': { borderColor: '#555' },
              '&:hover fieldset': { borderColor: '#777' },
              '&.Mui-focused fieldset': { borderColor: '#4fc3f7' },
            },
            input: { color: 'white' },
          }}
          InputProps={{
            startAdornment: <InputAdornment position="start"><Search sx={{ color: '#888' }} /></InputAdornment>,
            endAdornment: searchTerm && (
              <InputAdornment position="end">
                <IconButton onClick={() => setSearchTerm('')} size="small">
                  <Clear sx={{ color: '#888' }} />
                </IconButton>
              </InputAdornment>
            ),
          }}
        />
      </Box>

      <Box sx={{ overflowY: 'auto', p: 1, pt: 0 }}>
        {filteredState !== undefined ? (
          <JsonTreeView data={filteredState} defaultOpen={true} />
        ) : (
          <Typography sx={{ p: 2, color: '#888', textAlign: 'center' }}>No results</Typography>
        )}
      </Box>
    </Paper>}
    </>
  );
}

export default DevTools;