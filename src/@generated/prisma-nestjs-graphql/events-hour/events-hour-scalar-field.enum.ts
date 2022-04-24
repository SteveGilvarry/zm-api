import { registerEnumType } from '@nestjs/graphql';

export enum Events_HourScalarFieldEnum {
    EventId = "EventId",
    MonitorId = "MonitorId",
    StartDateTime = "StartDateTime",
    DiskSpace = "DiskSpace"
}


registerEnumType(Events_HourScalarFieldEnum, { name: 'Events_HourScalarFieldEnum', description: undefined })
