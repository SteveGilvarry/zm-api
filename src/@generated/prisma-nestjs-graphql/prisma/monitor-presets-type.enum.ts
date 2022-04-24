import { registerEnumType } from '@nestjs/graphql';

export enum MonitorPresets_Type {
    Local = "Local",
    Remote = "Remote",
    File = "File",
    Ffmpeg = "Ffmpeg",
    Libvlc = "Libvlc",
    cURL = "cURL",
    WebSite = "WebSite",
    NVSocket = "NVSocket"
}


registerEnumType(MonitorPresets_Type, { name: 'MonitorPresets_Type', description: undefined })
