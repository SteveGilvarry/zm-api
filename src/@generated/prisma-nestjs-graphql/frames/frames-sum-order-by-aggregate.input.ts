import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class FramesSumOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    EventId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FrameId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Delta?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Score?: keyof typeof SortOrder;
}
