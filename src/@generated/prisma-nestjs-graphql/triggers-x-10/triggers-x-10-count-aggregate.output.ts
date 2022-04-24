import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class TriggersX10CountAggregate {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    Activation!: number;

    @Field(() => Int, {nullable:false})
    AlarmInput!: number;

    @Field(() => Int, {nullable:false})
    AlarmOutput!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
