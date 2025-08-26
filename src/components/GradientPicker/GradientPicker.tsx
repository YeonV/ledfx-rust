import { useState, useRef, useEffect } from 'react'
import Popper from '@mui/material/Popper'
import ReactGPicker from 'react-gcolor-picker'
import { TextField, Button, useTheme, Typography } from '@mui/material'
import { Add } from '@mui/icons-material'
import useClickOutside from '../../utils/useClickOutside'
import Popover from '../Popover/Popover'
import { classes, Root } from './GradientPicker.styles'
import { GradientPickerProps } from './GradientPicker.props'

const GradientPicker = ({
  pickerBgColor = '#800000',
  title = 'Color',
  index = 1,
  isGradient = false,
  wrapperStyle = undefined,
  colors = undefined,
  handleAddGradient = undefined,
  sendColorToVirtuals = undefined,
  showHex = false
}: GradientPickerProps) => {
  const theme = useTheme()
  const popover = useRef(null)
  const [anchorEl, setAnchorEl] = useState(null)
  const [name, setName] = useState('')
  const [dialogOpen, setDialogOpen] = useState(false)
  const [pickerBgColorInt, setPickerBgColorInt] = useState(pickerBgColor)

  const defaultColors: any = {}
  if (colors?.gradients?.builtin)
    Object.entries(colors.gradients.builtin).forEach(([k, g]) => {
      defaultColors[k] = g
    })
  if (colors?.gradients?.user)
    Object.entries(colors.gradients.user)?.forEach(([k, g]) => {
      defaultColors[k] = g
    })
  if (colors?.colors?.builtin)
    Object.entries(colors.colors.builtin)?.forEach(([k, g]) => {
      defaultColors[k] = g
    })
  if (colors?.colors?.user)
    Object.entries(colors.colors.user)?.forEach(([k, g]) => {
      defaultColors[k] = g
    })

  const handleClick = (event: any) => {
    setAnchorEl(anchorEl ? null : event.currentTarget)
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  useClickOutside(popover, handleClose)

  const handleDeleteDialog = () => {
    setAnchorEl(null)
    setDialogOpen(true)
  }
  const open = Boolean(anchorEl)
  const id = open ? 'simple-popper' : undefined

  useEffect(() => {
    setPickerBgColorInt(pickerBgColor)
  }, [pickerBgColor, setPickerBgColorInt])

  useEffect(() => {
    setPickerBgColorInt(pickerBgColor)
  }, [pickerBgColor, setPickerBgColorInt])

  return (
    <Root
      className={`${classes.wrapper} step-effect-${index} gradient-picker`}
      style={{
        borderColor: theme.palette.divider,
        minWidth: 'unset',
        flexBasis: '49%',
        ...(wrapperStyle as any)
      }}
      // style={{
      //   ...wrapperStyle,
      //   '& > label': {
      //     backgroundColor: theme.palette.background.paper,
      //   },
      // }}
    >
      <Typography variant='caption' className="MuiFormLabel-root" style={{ background: theme.palette.background.paper }}>
        {title && title.replace(/_/g, ' ').replace(/background/g, 'bg').replace(/name/g, '')}
      </Typography>
      <div
        className={classes.picker}
        style={{ background: pickerBgColorInt }}
        aria-describedby={id}
        onClick={handleClick}
      />

      <Popper
        id={id}
        open={open}
        anchorEl={anchorEl}
        ref={popover && popover}
        sx={{ zIndex: 1300 }}
      >
        <div
          className={`${classes.paper} gradient-picker ${showHex ? 'show_hex' : ''}`}
          style={{
            padding: 8,
            backgroundColor: theme.palette.background.paper
            // '& .popup_tabs-header-label-active': {
            //   color: theme.palette.text.primary,
            // },
            // '& .popup_tabs-header-label': {
            //   color: theme.palette.text.disabled,
            //   '&.popup_tabs-header-label-active': {
            //     color: theme.palette.text.primary,
            //   },
            // },
          }}
        >
          <ReactGPicker
            colorBoardHeight={150}
            debounce
            debounceMS={300}
            format="hex"
            gradient={isGradient}
            solid
            onChange={(c) => {
              setPickerBgColorInt(c)
              return sendColorToVirtuals(c)
            }}
            popupWidth={288}
            showAlpha={false}
            value={pickerBgColorInt}
            defaultColors={Object.values(defaultColors)}
          />
          <div
            style={{
              marginTop: 2.5,
              width: '100%',
              display: 'flex',
              justifyContent: 'flex-end'
            }}
          >
            <Button
              style={{
                width: 69,
                height: 30,
                borderRadius: 4,
                border: '1px solid #999',
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                fontSize: 24,
                marginRight: 16,
                cursor: 'pointer'
              }}
              onClick={() => handleDeleteDialog()}
              disabled={
                colors &&
                colors.length &&
                colors.colors?.length &&
                colors.gradients?.length &&
                !(Object.keys(colors?.colors?.user)?.length > 0) &&
                !(Object.keys(colors?.gradients?.user)?.length > 0)
              }
            >
              -
            </Button>
            <Popover
              className={classes.addButton}
              popoverStyle={{ padding: '0.5rem' }}
              color="primary"
              content={
                <TextField
                  autoFocus
                  onClick={(e) => e.stopPropagation()}
                  onKeyDown={(e: any) => e.key === 'Enter' && handleAddGradient(name)}
                  error={
                    colors?.length &&
                    colors.colors?.length &&
                    colors.gradients?.length &&
                    (Object.keys(colors.colors).indexOf(name) > -1 ||
                      Object.values(colors.colors).filter((p) => p === pickerBgColorInt)?.length >
                        0 ||
                      Object.keys(colors.gradients).indexOf(name) > -1 ||
                      Object.values(colors.gradients).filter((p) => p === pickerBgColorInt).length >
                        0)
                  }
                  size="small"
                  id="gradientNameInput"
                  label="Enter name to save as..."
                  style={{ marginRight: '1rem', flex: 1 }}
                  value={name}
                  onChange={(e) => {
                    setName(e.target.value)
                  }}
                />
              }
              confirmDisabled={name.length === 0}
              onConfirm={() => handleAddGradient(name)}
              startIcon=""
              size="medium"
              icon={<Add />}
            />
          </div>
        </div>
      </Popper>
    </Root>
  )
}

export default GradientPicker
