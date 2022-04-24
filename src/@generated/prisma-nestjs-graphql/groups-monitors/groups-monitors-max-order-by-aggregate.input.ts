import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class Groups_MonitorsMaxOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    GroupId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;
}
