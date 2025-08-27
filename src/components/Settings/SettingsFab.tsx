import { useStore } from "../../store/useStore";
import { IconBtn } from "../IconBtn";
import { Settings } from "../Settings/Settings";
import { Settings as SettingsIcon } from "@mui/icons-material";
import { Drawer } from "@mui/material";

/**
 * Floating action button for opening the settings drawer.
 */
export function SettingsFab() {
  const { openSettings, setOpenSettings } = useStore();

  return (
    <>
      <IconBtn icon={<SettingsIcon />} text="Open Settings" onClick={() => setOpenSettings(true)} />
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
