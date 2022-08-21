import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { Events_Orientation } from './events-orientation.enum';
import { Events_Scheme } from '../prisma/events-scheme.enum';
import { EventsCountAggregate } from './events-count-aggregate.output';
import { EventsAvgAggregate } from './events-avg-aggregate.output';
import { EventsSumAggregate } from './events-sum-aggregate.output';
import { EventsMinAggregate } from './events-min-aggregate.output';
import { EventsMaxAggregate } from './events-max-aggregate.output';

@ObjectType()
export class EventsGroupBy {

    @Field(() => String, {nullable:false})
    Id!: bigint | number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    StorageId!: number;

    @Field(() => Int, {nullable:true})
    SecondaryStorageId?: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => String, {nullable:false})
    Cause!: string;

    @Field(() => Date, {nullable:true})
    StartDateTime?: Date | string;

    @Field(() => Date, {nullable:true})
    EndDateTime?: Date | string;

    @Field(() => Int, {nullable:false})
    Width!: number;

    @Field(() => Int, {nullable:false})
    Height!: number;

    @Field(() => GraphQLDecimal, {nullable:false})
    Length!: Decimal;

    @Field(() => Int, {nullable:true})
    Frames?: number;

    @Field(() => Int, {nullable:true})
    AlarmFrames?: number;

    @Field(() => String, {nullable:false})
    DefaultVideo!: string;

    @Field(() => Int, {nullable:true})
    SaveJPEGs?: number;

    @Field(() => Int, {nullable:false})
    TotScore!: number;

    @Field(() => Int, {nullable:true})
    AvgScore?: number;

    @Field(() => Int, {nullable:true})
    MaxScore?: number;

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

    @Field(() => String, {nullable:true})
    Notes?: string;

    @Field(() => Int, {nullable:false})
    StateId!: number;

    @Field(() => Events_Orientation, {nullable:false})
    Orientation!: keyof typeof Events_Orientation;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;

    @Field(() => Events_Scheme, {nullable:false})
    Scheme!: keyof typeof Events_Scheme;

    @Field(() => Boolean, {nullable:false})
    Locked!: boolean;

    @Field(() => EventsCountAggregate, {nullable:true})
    _count?: EventsCountAggregate;

    @Field(() => EventsAvgAggregate, {nullable:true})
    _avg?: EventsAvgAggregate;

    @Field(() => EventsSumAggregate, {nullable:true})
    _sum?: EventsSumAggregate;

    @Field(() => EventsMinAggregate, {nullable:true})
    _min?: EventsMinAggregate;

    @Field(() => EventsMaxAggregate, {nullable:true})
    _max?: EventsMaxAggregate;
}
