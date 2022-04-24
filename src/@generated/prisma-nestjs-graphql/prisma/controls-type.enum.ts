import { registerEnumType } from '@nestjs/graphql';

export enum Controls_Type {
    Local = "Local",
    Remote = "Remote",
    File = "File",
    Ffmpeg = "Ffmpeg",
    Libvlc = "Libvlc",
    cURL = "cURL",
    WebSite = "WebSite",
    NVSocket = "NVSocket"
}


registerEnumType(Controls_Type, { name: 'Controls_Type', description: undefined })
