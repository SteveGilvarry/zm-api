import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class TriggersX10SumOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;
}
