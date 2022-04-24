import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class MonitorPresetsCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Type!: number;

    @Field(() => Int, {nullable:false})
    Device!: number;

    @Field(() => Int, {nullable:false})
    Channel!: number;

    @Field(() => Int, {nullable:false})
    Format!: number;

    @Field(() => Int, {nullable:false})
    Protocol!: number;

    @Field(() => Int, {nullable:false})
    Method!: number;

    @Field(() => Int, {nullable:false})
    Host!: number;

    @Field(() => Int, {nullable:false})
    Port!: number;

    @Field(() => Int, {nullable:false})
    Path!: number;

    @Field(() => Int, {nullable:false})
    SubPath!: number;

    @Field(() => Int, {nullable:false})
    Width!: number;

    @Field(() => Int, {nullable:false})
    Height!: number;

    @Field(() => Int, {nullable:false})
    Palette!: number;

    @Field(() => Int, {nullable:false})
    MaxFPS!: number;

    @Field(() => Int, {nullable:false})
    Controllable!: number;

    @Field(() => Int, {nullable:false})
    ControlId!: number;

    @Field(() => Int, {nullable:false})
    ControlDevice!: number;

    @Field(() => Int, {nullable:false})
    ControlAddress!: number;

    @Field(() => Int, {nullable:false})
    DefaultRate!: number;

    @Field(() => Int, {nullable:false})
    DefaultScale!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
