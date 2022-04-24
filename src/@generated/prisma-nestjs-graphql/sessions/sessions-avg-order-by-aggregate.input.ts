import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class SessionsAvgOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    access?: keyof typeof SortOrder;
}
