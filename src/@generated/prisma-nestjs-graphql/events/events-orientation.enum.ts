import { registerEnumType } from '@nestjs/graphql';

export enum Events_Orientation {
    ROTATE_0 = "ROTATE_0",
    ROTATE_90 = "ROTATE_90",
    ROTATE_180 = "ROTATE_180",
    ROTATE_270 = "ROTATE_270",
    FLIP_HORI = "FLIP_HORI",
    FLIP_VERT = "FLIP_VERT"
}


registerEnumType(Events_Orientation, { name: 'Events_Orientation', description: undefined })
