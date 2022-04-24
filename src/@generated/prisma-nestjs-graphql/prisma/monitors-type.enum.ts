import { registerEnumType } from '@nestjs/graphql';

export enum Monitors_Type {
    Local = "Local",
    Remote = "Remote",
    File = "File",
    Ffmpeg = "Ffmpeg",
    Libvlc = "Libvlc",
    cURL = "cURL",
    NVSocket = "NVSocket",
    VNC = "VNC"
}


registerEnumType(Monitors_Type, { name: 'Monitors_Type', description: undefined })
