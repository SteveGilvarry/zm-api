import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class ManufacturersSumOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;
}
