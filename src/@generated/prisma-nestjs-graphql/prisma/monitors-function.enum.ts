import { registerEnumType } from '@nestjs/graphql';

export enum Monitors_Function {
    None = "None",
    Monitor = "Monitor",
    Modect = "Modect",
    Record = "Record",
    Mocord = "Mocord",
    Nodect = "Nodect"
}


registerEnumType(Monitors_Function, { name: 'Monitors_Function', description: undefined })
