import { registerEnumType } from '@nestjs/graphql';

export enum ZonePresetsScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    Type = "Type",
    Units = "Units",
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


registerEnumType(ZonePresetsScalarFieldEnum, { name: 'ZonePresetsScalarFieldEnum', description: undefined })
