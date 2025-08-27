import { useStore } from "../../store/useStore";
import { WledDiscoverer } from "../WledDiscoverer";
import { AddButton } from "../AddButton";
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