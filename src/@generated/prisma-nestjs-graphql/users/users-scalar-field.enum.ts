import { registerEnumType } from '@nestjs/graphql';

export enum UsersScalarFieldEnum {
    Id = "Id",
    Username = "Username",
    Password = "Password",
    Language = "Language",
    Enabled = "Enabled",
    Stream = "Stream",
    Events = "Events",
    Control = "Control",
    Monitors = "Monitors",
    Groups = "Groups",
    Devices = "Devices",
    Snapshots = "Snapshots",
    System = "System",
    MaxBandwidth = "MaxBandwidth",
    MonitorIds = "MonitorIds",
    TokenMinExpiry = "TokenMinExpiry",
    APIEnabled = "APIEnabled",
    HomeView = "HomeView"
}


registerEnumType(UsersScalarFieldEnum, { name: 'UsersScalarFieldEnum', description: undefined })
