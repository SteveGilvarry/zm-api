import { registerEnumType } from '@nestjs/graphql';

export enum Zones_CheckMethod {
    AlarmedPixels = "AlarmedPixels",
    FilteredPixels = "FilteredPixels",
    Blobs = "Blobs"
}


registerEnumType(Zones_CheckMethod, { name: 'Zones_CheckMethod', description: undefined })
