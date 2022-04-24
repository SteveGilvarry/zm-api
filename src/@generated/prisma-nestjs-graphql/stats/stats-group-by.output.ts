import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { StatsCountAggregate } from './stats-count-aggregate.output';
import { StatsAvgAggregate } from './stats-avg-aggregate.output';
import { StatsSumAggregate } from './stats-sum-aggregate.output';
import { StatsMinAggregate } from './stats-min-aggregate.output';
import { StatsMaxAggregate } from './stats-max-aggregate.output';

@ObjectType()
export class StatsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    ZoneId!: number;

    @Field(() => String, {nullable:false})
    EventId!: bigint | number;

    @Field(() => Int, {nullable:false})
    FrameId!: number;

    @Field(() => Int, {nullable:false})
    PixelDiff!: number;

    @Field(() => Int, {nullable:false})
    AlarmPixels!: number;

    @Field(() => Int, {nullable:false})
    FilterPixels!: number;

    @Field(() => Int, {nullable:false})
    BlobPixels!: number;

    @Field(() => Int, {nullable:false})
    Blobs!: number;

    @Field(() => Int, {nullable:false})
    MinBlobSize!: number;

    @Field(() => Int, {nullable:false})
    MaxBlobSize!: number;

    @Field(() => Int, {nullable:false})
    MinX!: number;

    @Field(() => Int, {nullable:false})
    MaxX!: number;

    @Field(() => Int, {nullable:false})
    MinY!: number;

    @Field(() => Int, {nullable:false})
    MaxY!: number;

    @Field(() => Int, {nullable:false})
    Score!: number;

    @Field(() => StatsCountAggregate, {nullable:true})
    _count?: StatsCountAggregate;

    @Field(() => StatsAvgAggregate, {nullable:true})
    _avg?: StatsAvgAggregate;

    @Field(() => StatsSumAggregate, {nullable:true})
    _sum?: StatsSumAggregate;

    @Field(() => StatsMinAggregate, {nullable:true})
    _min?: StatsMinAggregate;

    @Field(() => StatsMaxAggregate, {nullable:true})
    _max?: StatsMaxAggregate;
}
