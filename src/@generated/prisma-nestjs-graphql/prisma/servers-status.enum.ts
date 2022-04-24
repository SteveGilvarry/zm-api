import { registerEnumType } from '@nestjs/graphql';

export enum Servers_Status {
    Unknown = "Unknown",
    NotRunning = "NotRunning",
    Running = "Running"
}


registerEnumType(Servers_Status, { name: 'Servers_Status', description: undefined })
