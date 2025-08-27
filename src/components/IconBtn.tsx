
import { IconButton, IconButtonProps, Tooltip } from "@mui/material";

type IconBtnProps = {
  icon: React.ReactNode;
  text: string;
} & IconButtonProps;

const IconBtn = ({icon, text, ...props}: IconBtnProps) => {
  return (
    <Tooltip title={text}>
      <IconButton {...props}>{icon}</IconButton>
    </Tooltip>
  )
}

export default IconBtn;