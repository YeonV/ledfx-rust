import { AppBar, Toolbar } from "@mui/material";
import { LeftActions } from "./LeftActions";
import { RightActions } from "./RightActions";

function TopBar() {
  return (
    <AppBar elevation={0} color="error" position="sticky">
      <Toolbar color="error" sx={{ minHeight: '48px !important', justifyContent: 'space-between', px: '16px !important' }}>
        <LeftActions />
        <RightActions />
      </Toolbar>
    </AppBar>
  );
}

export default TopBar;