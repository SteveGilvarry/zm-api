import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class TriggersX10MaxAggregate {

    @Field(() => Int, {nullable:true})
    MonitorId?: number;

    @Field(() => String, {nullable:true})
    Activation?: string;

    @Field(() => String, {nullable:true})
    AlarmInput?: string;

    @Field(() => String, {nullable:true})
    AlarmOutput?: string;
}
