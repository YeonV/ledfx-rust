import { useStore } from "../../store/useStore";
import { Settings } from "../Settings/Settings";
import { Equalizer } from "@mui/icons-material";
import { Drawer, IconButton } from "@mui/material";
import MelbankVisualizer from "./MelbankVisualizer";

/**
 * Floating action button for opening the settings drawer.
 */
export function MelbankVisualizerFab() {
  const { openMelbankVisualizer, setOpenMelbankVisualizer } = useStore();

  return (
    <>
        <IconButton onClick={() => setOpenMelbankVisualizer(true)} sx={{ position: "fixed", top: 16, right: 48 }}>
          <Equalizer />
        </IconButton>
        <Drawer
          open={openMelbankVisualizer}
          onClose={() => setOpenMelbankVisualizer(false)}
          anchor="bottom"
        >
          <MelbankVisualizer />
        </Drawer>
    </>
  );
}
