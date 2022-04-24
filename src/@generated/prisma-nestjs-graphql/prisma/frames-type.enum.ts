import { registerEnumType } from '@nestjs/graphql';

export enum Frames_Type {
    Normal = "Normal",
    Bulk = "Bulk",
    Alarm = "Alarm"
}


registerEnumType(Frames_Type, { name: 'Frames_Type', description: undefined })
