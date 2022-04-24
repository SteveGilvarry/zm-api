import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class Events_ArchivedMaxOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;
}
