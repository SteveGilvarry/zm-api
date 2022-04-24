import { registerEnumType } from '@nestjs/graphql';

export enum Events_DayScalarFieldEnum {
    EventId = "EventId",
    MonitorId = "MonitorId",
    StartDateTime = "StartDateTime",
    DiskSpace = "DiskSpace"
}


registerEnumType(Events_DayScalarFieldEnum, { name: 'Events_DayScalarFieldEnum', description: undefined })
