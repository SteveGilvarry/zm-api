import { registerEnumType } from '@nestjs/graphql';

export enum ServersScalarFieldEnum {
    Id = "Id",
    Protocol = "Protocol",
    Hostname = "Hostname",
    Port = "Port",
    PathToIndex = "PathToIndex",
    PathToZMS = "PathToZMS",
    PathToApi = "PathToApi",
    Name = "Name",
    State_Id = "State_Id",
    Status = "Status",
    CpuLoad = "CpuLoad",
    TotalMem = "TotalMem",
    FreeMem = "FreeMem",
    TotalSwap = "TotalSwap",
    FreeSwap = "FreeSwap",
    zmstats = "zmstats",
    zmaudit = "zmaudit",
    zmtrigger = "zmtrigger",
    zmeventnotification = "zmeventnotification"
}


registerEnumType(ServersScalarFieldEnum, { name: 'ServersScalarFieldEnum', description: undefined })
