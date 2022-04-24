import { registerEnumType } from '@nestjs/graphql';

export enum Monitors_OutputContainer {
    auto = "auto",
    mp4 = "mp4",
    mkv = "mkv"
}


registerEnumType(Monitors_OutputContainer, { name: 'Monitors_OutputContainer', description: undefined })
