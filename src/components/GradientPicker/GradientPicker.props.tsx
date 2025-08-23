import { CSSProperties } from "@mui/material"

export interface GradientPickerProps {
  pickerBgColor: string
  title?: string
  index?: number
  isGradient?: boolean
  wrapperStyle?: CSSProperties
  colors?: any
  handleAddGradient?: any
  sendColorToVirtuals?: any
  showHex?: boolean
}
