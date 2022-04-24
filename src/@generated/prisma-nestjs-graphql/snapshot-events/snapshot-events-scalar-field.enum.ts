import { registerEnumType } from '@nestjs/graphql';

export enum Snapshot_EventsScalarFieldEnum {
    Id = "Id",
    SnapshotId = "SnapshotId",
    EventId = "EventId"
}


registerEnumType(Snapshot_EventsScalarFieldEnum, { name: 'Snapshot_EventsScalarFieldEnum', description: undefined })
