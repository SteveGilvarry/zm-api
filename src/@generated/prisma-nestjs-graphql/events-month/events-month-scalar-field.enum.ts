import { registerEnumType } from '@nestjs/graphql';

export enum Events_MonthScalarFieldEnum {
    EventId = "EventId",
    MonitorId = "MonitorId",
    StartDateTime = "StartDateTime",
    DiskSpace = "DiskSpace"
}


registerEnumType(Events_MonthScalarFieldEnum, { name: 'Events_MonthScalarFieldEnum', description: undefined })
