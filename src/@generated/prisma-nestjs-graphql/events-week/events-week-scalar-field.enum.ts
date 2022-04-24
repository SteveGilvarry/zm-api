import { registerEnumType } from '@nestjs/graphql';

export enum Events_WeekScalarFieldEnum {
    EventId = "EventId",
    MonitorId = "MonitorId",
    StartDateTime = "StartDateTime",
    DiskSpace = "DiskSpace"
}


registerEnumType(Events_WeekScalarFieldEnum, { name: 'Events_WeekScalarFieldEnum', description: undefined })
