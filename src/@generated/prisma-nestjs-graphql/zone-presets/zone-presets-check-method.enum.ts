import { registerEnumType } from '@nestjs/graphql';

export enum ZonePresets_CheckMethod {
    AlarmedPixels = "AlarmedPixels",
    FilteredPixels = "FilteredPixels",
    Blobs = "Blobs"
}


registerEnumType(ZonePresets_CheckMethod, { name: 'ZonePresets_CheckMethod', description: undefined })
