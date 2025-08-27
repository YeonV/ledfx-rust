import { useStore } from "../../store/useStore";
import { Equalizer } from "@mui/icons-material";
import { Drawer } from "@mui/material";
import { IconBtn } from "../IconBtn";
import MelbankVisualizer from "./MelbankVisualizer";

/**
 * Floating action button for opening the settings drawer.
 */
export function MelbankVisualizerFab() {
  const { openMelbankVisualizer, setOpenMelbankVisualizer } = useStore();

  return (
    <>
        <IconBtn icon={<Equalizer />} text="Open Melbank Visualizer" onClick={() => setOpenMelbankVisualizer(true)} />
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
