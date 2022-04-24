import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class ServersSumOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Port?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    State_Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CpuLoad?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotalMem?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FreeMem?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotalSwap?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FreeSwap?: keyof typeof SortOrder;
}
