import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class EventsCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    StorageId!: number;

    @Field(() => Int, {nullable:false})
    SecondaryStorageId!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Cause!: number;

    @Field(() => Int, {nullable:false})
    StartDateTime!: number;

    @Field(() => Int, {nullable:false})
    EndDateTime!: number;

    @Field(() => Int, {nullable:false})
    Width!: number;

    @Field(() => Int, {nullable:false})
    Height!: number;

    @Field(() => Int, {nullable:false})
    Length!: number;

    @Field(() => Int, {nullable:false})
    Frames!: number;

    @Field(() => Int, {nullable:false})
    AlarmFrames!: number;

    @Field(() => Int, {nullable:false})
    DefaultVideo!: number;

    @Field(() => Int, {nullable:false})
    SaveJPEGs!: number;

    @Field(() => Int, {nullable:false})
    TotScore!: number;

    @Field(() => Int, {nullable:false})
    AvgScore!: number;

    @Field(() => Int, {nullable:false})
    MaxScore!: number;

    @Field(() => Int, {nullable:false})
    Archived!: number;

    @Field(() => Int, {nullable:false})
    Videoed!: number;

    @Field(() => Int, {nullable:false})
    Uploaded!: number;

    @Field(() => Int, {nullable:false})
    Emailed!: number;

    @Field(() => Int, {nullable:false})
    Messaged!: number;

    @Field(() => Int, {nullable:false})
    Executed!: number;

    @Field(() => Int, {nullable:false})
    Notes!: number;

    @Field(() => Int, {nullable:false})
    StateId!: number;

    @Field(() => Int, {nullable:false})
    Orientation!: number;

    @Field(() => Int, {nullable:false})
    DiskSpace!: number;

    @Field(() => Int, {nullable:false})
    Scheme!: number;

    @Field(() => Int, {nullable:false})
    Locked!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
