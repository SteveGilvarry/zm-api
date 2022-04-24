import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ControlPresetsCountAggregate {

    @Field(() => Int, {nullable:false})
    MonitorId!: number;

    @Field(() => Int, {nullable:false})
    Preset!: number;

    @Field(() => Int, {nullable:false})
    Label!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
