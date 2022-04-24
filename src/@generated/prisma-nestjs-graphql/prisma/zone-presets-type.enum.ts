import { registerEnumType } from '@nestjs/graphql';

export enum ZonePresets_Type {
    Active = "Active",
    Inclusive = "Inclusive",
    Exclusive = "Exclusive",
    Preclusive = "Preclusive",
    Inactive = "Inactive",
    Privacy = "Privacy"
}


registerEnumType(ZonePresets_Type, { name: 'ZonePresets_Type', description: undefined })
