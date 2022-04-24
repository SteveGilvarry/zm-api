import { registerEnumType } from '@nestjs/graphql';

export enum Monitor_Status_Status {
    Unknown = "Unknown",
    NotRunning = "NotRunning",
    Running = "Running",
    Connected = "Connected",
    Signal = "Signal"
}


registerEnumType(Monitor_Status_Status, { name: 'Monitor_Status_Status', description: undefined })
