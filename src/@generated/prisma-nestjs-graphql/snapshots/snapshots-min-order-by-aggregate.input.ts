import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class SnapshotsMinOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Description?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CreatedBy?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CreatedOn?: keyof typeof SortOrder;
}
