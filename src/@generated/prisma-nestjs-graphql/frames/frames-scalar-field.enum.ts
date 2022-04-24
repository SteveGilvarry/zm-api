import { registerEnumType } from '@nestjs/graphql';

export enum FramesScalarFieldEnum {
    Id = "Id",
    EventId = "EventId",
    FrameId = "FrameId",
    Type = "Type",
    TimeStamp = "TimeStamp",
    Delta = "Delta",
    Score = "Score"
}


registerEnumType(FramesScalarFieldEnum, { name: 'FramesScalarFieldEnum', description: undefined })
