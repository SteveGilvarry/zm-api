import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class StorageCountOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Path?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Url?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Scheme?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ServerId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DoDelete?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Enabled?: keyof typeof SortOrder;
}
