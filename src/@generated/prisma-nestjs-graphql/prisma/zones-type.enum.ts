import { registerEnumType } from '@nestjs/graphql';

export enum Zones_Type {
    Active = "Active",
    Inclusive = "Inclusive",
    Exclusive = "Exclusive",
    Preclusive = "Preclusive",
    Inactive = "Inactive",
    Privacy = "Privacy"
}


registerEnumType(Zones_Type, { name: 'Zones_Type', description: undefined })
