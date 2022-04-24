import { registerEnumType } from '@nestjs/graphql';

export enum Devices_Type {
    X10 = "X10"
}


registerEnumType(Devices_Type, { name: 'Devices_Type', description: undefined })
