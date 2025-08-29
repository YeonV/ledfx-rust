import { useStore } from "../../store/useStore";
import { WledDiscoverer } from "../WLED/WledDiscoverer";
import { AddButton } from "../Virtuals/AddButton";
import { Box } from "@mui/material";

export function LeftActions() {
    const { devices } = useStore();

    return (
        <Box>
            <WledDiscoverer />
            {devices.length > 0 && <AddButton />}
        </Box>
    );
}