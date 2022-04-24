import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class Events_HourAvgAggregate {

    @Field(() => Float, {nullable:true})
    EventId?: number;

    @Field(() => Float, {nullable:true})
    MonitorId?: number;

    @Field(() => Float, {nullable:true})
    DiskSpace?: number;
}
