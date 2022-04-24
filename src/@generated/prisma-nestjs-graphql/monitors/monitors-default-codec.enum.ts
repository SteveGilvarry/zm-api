import { registerEnumType } from '@nestjs/graphql';

export enum Monitors_DefaultCodec {
    auto = "auto",
    MP4 = "MP4",
    MJPEG = "MJPEG"
}


registerEnumType(Monitors_DefaultCodec, { name: 'Monitors_DefaultCodec', description: undefined })
