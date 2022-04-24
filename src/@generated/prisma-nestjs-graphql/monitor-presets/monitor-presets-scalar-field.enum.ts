import { registerEnumType } from '@nestjs/graphql';

export enum MonitorPresetsScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    Type = "Type",
    Device = "Device",
    Channel = "Channel",
    Format = "Format",
    Protocol = "Protocol",
    Method = "Method",
    Host = "Host",
    Port = "Port",
    Path = "Path",
    SubPath = "SubPath",
    Width = "Width",
    Height = "Height",
    Palette = "Palette",
    MaxFPS = "MaxFPS",
    Controllable = "Controllable",
    ControlId = "ControlId",
    ControlDevice = "ControlDevice",
    ControlAddress = "ControlAddress",
    DefaultRate = "DefaultRate",
    DefaultScale = "DefaultScale"
}


registerEnumType(MonitorPresetsScalarFieldEnum, { name: 'MonitorPresetsScalarFieldEnum', description: undefined })
