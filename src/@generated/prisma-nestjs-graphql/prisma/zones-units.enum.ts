import { registerEnumType } from '@nestjs/graphql';

export enum Zones_Units {
    Pixels = "Pixels",
    Percent = "Percent"
}


registerEnumType(Zones_Units, { name: 'Zones_Units', description: undefined })
