import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class Events_ArchivedMaxAggregateInput {

    @Field(() => Boolean, {nullable:true})
    EventId?: true;

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    DiskSpace?: true;
}