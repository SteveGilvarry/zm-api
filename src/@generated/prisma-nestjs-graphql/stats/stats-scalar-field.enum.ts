import { registerEnumType } from '@nestjs/graphql';

export enum StatsScalarFieldEnum {
    Id = "Id",
    MonitorId = "MonitorId",
    ZoneId = "ZoneId",
    EventId = "EventId",
    FrameId = "FrameId",
    PixelDiff = "PixelDiff",
    AlarmPixels = "AlarmPixels",
    FilterPixels = "FilterPixels",
    BlobPixels = "BlobPixels",
    Blobs = "Blobs",
    MinBlobSize = "MinBlobSize",
    MaxBlobSize = "MaxBlobSize",
    MinX = "MinX",
    MaxX = "MaxX",
    MinY = "MinY",
    MaxY = "MaxY",
    Score = "Score"
}


registerEnumType(StatsScalarFieldEnum, { name: 'StatsScalarFieldEnum', description: undefined })
