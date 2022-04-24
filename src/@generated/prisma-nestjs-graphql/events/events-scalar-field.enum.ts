import { registerEnumType } from '@nestjs/graphql';

export enum EventsScalarFieldEnum {
    Id = "Id",
    MonitorId = "MonitorId",
    StorageId = "StorageId",
    SecondaryStorageId = "SecondaryStorageId",
    Name = "Name",
    Cause = "Cause",
    StartDateTime = "StartDateTime",
    EndDateTime = "EndDateTime",
    Width = "Width",
    Height = "Height",
    Length = "Length",
    Frames = "Frames",
    AlarmFrames = "AlarmFrames",
    DefaultVideo = "DefaultVideo",
    SaveJPEGs = "SaveJPEGs",
    TotScore = "TotScore",
    AvgScore = "AvgScore",
    MaxScore = "MaxScore",
    Archived = "Archived",
    Videoed = "Videoed",
    Uploaded = "Uploaded",
    Emailed = "Emailed",
    Messaged = "Messaged",
    Executed = "Executed",
    Notes = "Notes",
    StateId = "StateId",
    Orientation = "Orientation",
    DiskSpace = "DiskSpace",
    Scheme = "Scheme",
    Locked = "Locked"
}


registerEnumType(EventsScalarFieldEnum, { name: 'EventsScalarFieldEnum', description: undefined })
