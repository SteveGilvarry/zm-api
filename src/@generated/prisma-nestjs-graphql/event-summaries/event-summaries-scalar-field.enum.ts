import { registerEnumType } from '@nestjs/graphql';

export enum Event_SummariesScalarFieldEnum {
    MonitorId = "MonitorId",
    TotalEvents = "TotalEvents",
    TotalEventDiskSpace = "TotalEventDiskSpace",
    HourEvents = "HourEvents",
    HourEventDiskSpace = "HourEventDiskSpace",
    DayEvents = "DayEvents",
    DayEventDiskSpace = "DayEventDiskSpace",
    WeekEvents = "WeekEvents",
    WeekEventDiskSpace = "WeekEventDiskSpace",
    MonthEvents = "MonthEvents",
    MonthEventDiskSpace = "MonthEventDiskSpace",
    ArchivedEvents = "ArchivedEvents",
    ArchivedEventDiskSpace = "ArchivedEventDiskSpace"
}


registerEnumType(Event_SummariesScalarFieldEnum, { name: 'Event_SummariesScalarFieldEnum', description: undefined })
