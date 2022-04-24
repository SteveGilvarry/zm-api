import { registerEnumType } from '@nestjs/graphql';

export enum ZonesScalarFieldEnum {
    Id = "Id",
    MonitorId = "MonitorId",
    Name = "Name",
    Type = "Type",
    Units = "Units",
    NumCoords = "NumCoords",
    Coords = "Coords",
    Area = "Area",
    AlarmRGB = "AlarmRGB",
    CheckMethod = "CheckMethod",
    MinPixelThreshold = "MinPixelThreshold",
    MaxPixelThreshold = "MaxPixelThreshold",
    MinAlarmPixels = "MinAlarmPixels",
    MaxAlarmPixels = "MaxAlarmPixels",
    FilterX = "FilterX",
    FilterY = "FilterY",
    MinFilterPixels = "MinFilterPixels",
    MaxFilterPixels = "MaxFilterPixels",
    MinBlobPixels = "MinBlobPixels",
    MaxBlobPixels = "MaxBlobPixels",
    MinBlobs = "MinBlobs",
    MaxBlobs = "MaxBlobs",
    OverloadFrames = "OverloadFrames",
    ExtendAlarmFrames = "ExtendAlarmFrames"
}


registerEnumType(ZonesScalarFieldEnum, { name: 'ZonesScalarFieldEnum', description: undefined })
