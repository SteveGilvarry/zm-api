import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { ConfigCountAggregate } from './config-count-aggregate.output';
import { ConfigAvgAggregate } from './config-avg-aggregate.output';
import { ConfigSumAggregate } from './config-sum-aggregate.output';
import { ConfigMinAggregate } from './config-min-aggregate.output';
import { ConfigMaxAggregate } from './config-max-aggregate.output';

@ObjectType()
export class ConfigGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => String, {nullable:false})
    Value!: string;

    @Field(() => String, {nullable:false})
    Type!: string;

    @Field(() => String, {nullable:true})
    DefaultValue?: string;

    @Field(() => String, {nullable:true})
    Hint?: string;

    @Field(() => String, {nullable:true})
    Pattern?: string;

    @Field(() => String, {nullable:true})
    Format?: string;

    @Field(() => String, {nullable:true})
    Prompt?: string;

    @Field(() => String, {nullable:true})
    Help?: string;

    @Field(() => String, {nullable:false})
    Category!: string;

    @Field(() => Int, {nullable:false})
    Readonly!: number;

    @Field(() => String, {nullable:true})
    Requires?: string;

    @Field(() => ConfigCountAggregate, {nullable:true})
    _count?: ConfigCountAggregate;

    @Field(() => ConfigAvgAggregate, {nullable:true})
    _avg?: ConfigAvgAggregate;

    @Field(() => ConfigSumAggregate, {nullable:true})
    _sum?: ConfigSumAggregate;

    @Field(() => ConfigMinAggregate, {nullable:true})
    _min?: ConfigMinAggregate;

    @Field(() => ConfigMaxAggregate, {nullable:true})
    _max?: ConfigMaxAggregate;
}
