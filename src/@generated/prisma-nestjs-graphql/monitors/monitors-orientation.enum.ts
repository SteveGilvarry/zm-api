import { registerEnumType } from '@nestjs/graphql';

export enum Monitors_Orientation {
    ROTATE_0 = "ROTATE_0",
    ROTATE_90 = "ROTATE_90",
    ROTATE_180 = "ROTATE_180",
    ROTATE_270 = "ROTATE_270",
    FLIP_HORI = "FLIP_HORI",
    FLIP_VERT = "FLIP_VERT"
}


registerEnumType(Monitors_Orientation, { name: 'Monitors_Orientation', description: undefined })
