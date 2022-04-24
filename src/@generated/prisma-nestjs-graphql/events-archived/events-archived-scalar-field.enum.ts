import { registerEnumType } from '@nestjs/graphql';

export enum Events_ArchivedScalarFieldEnum {
    EventId = "EventId",
    MonitorId = "MonitorId",
    DiskSpace = "DiskSpace"
}


registerEnumType(Events_ArchivedScalarFieldEnum, { name: 'Events_ArchivedScalarFieldEnum', description: undefined })
