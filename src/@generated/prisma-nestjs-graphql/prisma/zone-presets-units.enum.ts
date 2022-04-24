import { registerEnumType } from '@nestjs/graphql';

export enum ZonePresets_Units {
    Pixels = "Pixels",
    Percent = "Percent"
}


registerEnumType(ZonePresets_Units, { name: 'ZonePresets_Units', description: undefined })
