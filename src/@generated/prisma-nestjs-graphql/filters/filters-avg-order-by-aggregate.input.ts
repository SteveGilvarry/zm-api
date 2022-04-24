import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class FiltersAvgOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    UserId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoArchive?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoUnarchive?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoVideo?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoUpload?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoEmail?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoMessage?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoExecute?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoDelete?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoMove?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoCopy?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoCopyTo?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoMoveTo?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    UpdateDiskSpace?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Background?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Concurrent?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    LockRows?: keyof typeof SortOrder;
}
