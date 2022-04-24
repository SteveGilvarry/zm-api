import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ControlPresetsSumAggregate {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => Int, {nullable:true})
    Preset?: number;
}
