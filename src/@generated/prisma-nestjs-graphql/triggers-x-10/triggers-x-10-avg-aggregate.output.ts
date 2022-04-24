import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class TriggersX10AvgAggregate {

    @Field(() => Float, {nullable:true})
    MonitorId?: number;
}
