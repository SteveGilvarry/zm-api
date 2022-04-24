import { registerEnumType } from '@nestjs/graphql';

export enum Monitors_Importance {
    Not = "Not",
    Less = "Less",
    Normal = "Normal"
}


registerEnumType(Monitors_Importance, { name: 'Monitors_Importance', description: undefined })
