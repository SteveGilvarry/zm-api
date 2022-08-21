import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { MonitorPresets_Type } from '../prisma/monitor-presets-type.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { MonitorPresetsCountAggregate } from './monitor-presets-count-aggregate.output';
import { MonitorPresetsAvgAggregate } from './monitor-presets-avg-aggregate.output';
import { MonitorPresetsSumAggregate } from './monitor-presets-sum-aggregate.output';
import { MonitorPresetsMinAggregate } from './monitor-presets-min-aggregate.output';
import { MonitorPresetsMaxAggregate } from './monitor-presets-max-aggregate.output';

@ObjectType()
export class MonitorPresetsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => MonitorPresets_Type, {nullable:false})
    Type!: keyof typeof MonitorPresets_Type;

    @Field(() => String, {nullable:true})
    Device?: string;

    @Field(() => String, {nullable:true})
    Channel?: string;

    @Field(() => Int, {nullable:true})
    Format?: number;

    @Field(() => String, {nullable:true})
    Protocol?: string;

    @Field(() => String, {nullable:true})
    Method?: string;

    @Field(() => String, {nullable:true})
    Host?: string;

    @Field(() => String, {nullable:true})
    Port?: string;

    @Field(() => String, {nullable:true})
    Path?: string;

    @Field(() => String, {nullable:true})
    SubPath?: string;

    @Field(() => Int, {nullable:true})
    Width?: number;

    @Field(() => Int, {nullable:true})
    Height?: number;

    @Field(() => Int, {nullable:true})
    Palette?: number;

    @Field(() => GraphQLDecimal, {nullable:true})
    MaxFPS?: Decimal;

    @Field(() => Int, {nullable:false})
    Controllable!: number;

    @Field(() => String, {nullable:true})
    ControlId?: string;

    @Field(() => String, {nullable:true})
    ControlDevice?: string;

    @Field(() => String, {nullable:true})
    ControlAddress?: string;

    @Field(() => Int, {nullable:false})
    DefaultRate!: number;

    @Field(() => Int, {nullable:false})
    DefaultScale!: number;

    @Field(() => MonitorPresetsCountAggregate, {nullable:true})
    _count?: MonitorPresetsCountAggregate;

    @Field(() => MonitorPresetsAvgAggregate, {nullable:true})
    _avg?: MonitorPresetsAvgAggregate;

    @Field(() => MonitorPresetsSumAggregate, {nullable:true})
    _sum?: MonitorPresetsSumAggregate;

    @Field(() => MonitorPresetsMinAggregate, {nullable:true})
    _min?: MonitorPresetsMinAggregate;

    @Field(() => MonitorPresetsMaxAggregate, {nullable:true})
    _max?: MonitorPresetsMaxAggregate;
}
