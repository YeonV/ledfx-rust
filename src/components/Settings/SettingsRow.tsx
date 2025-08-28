import { Stack, Typography, Card, CardContent } from "@mui/material";

export function SettingsRow({ icon, title, children }: {
    icon?: React.ReactNode;
    title: string;
    children: React.ReactNode;
}) {

    return (
        
            <Card variant="outlined">
                <CardContent>
                    <Stack direction={'row'} justifyContent="space-between" alignItems="center">
                        <Stack direction="row" alignItems="center" spacing={1} width={'400px'}>
                            {icon}
                            <Typography variant="body2" pl={1}>{title}</Typography>
                        </Stack>
                        {children}
                    </Stack>
                </CardContent>
            </Card>
    );
}
