import { useStore } from "../../store/useStore";
import { Settings } from "../Settings/Settings";
import { Settings as SettingsIcon } from "@mui/icons-material";
import { Drawer, IconButton } from "@mui/material";

/**
 * Floating action button for opening the settings drawer.
 */
export function SettingsFab() {
  const { openSettings, setOpenSettings } = useStore();

  return (
    <>
        <IconButton onClick={() => setOpenSettings(true)} sx={{ position: "fixed", top: 16, right: 16 }}>
          <SettingsIcon />
        </IconButton>
        <Drawer
          open={openSettings}
          onClose={() => setOpenSettings(false)}
          anchor="bottom"
        >
          <Settings />
        </Drawer>
    </>
  );
}
