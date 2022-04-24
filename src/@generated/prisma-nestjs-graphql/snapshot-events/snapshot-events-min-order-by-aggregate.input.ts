import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class Snapshot_EventsMinOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SnapshotId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;
}
