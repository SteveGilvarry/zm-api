import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { FiltersCountOrderByAggregateInput } from './filters-count-order-by-aggregate.input';
import { FiltersAvgOrderByAggregateInput } from './filters-avg-order-by-aggregate.input';
import { FiltersMaxOrderByAggregateInput } from './filters-max-order-by-aggregate.input';
import { FiltersMinOrderByAggregateInput } from './filters-min-order-by-aggregate.input';
import { FiltersSumOrderByAggregateInput } from './filters-sum-order-by-aggregate.input';

@InputType()
export class FiltersOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    UserId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Query_json?: keyof typeof SortOrder;

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
    EmailTo?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    EmailSubject?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    EmailBody?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoMessage?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoExecute?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AutoExecuteCmd?: keyof typeof SortOrder;

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

    @Field(() => FiltersCountOrderByAggregateInput, {nullable:true})
    _count?: FiltersCountOrderByAggregateInput;

    @Field(() => FiltersAvgOrderByAggregateInput, {nullable:true})
    _avg?: FiltersAvgOrderByAggregateInput;

    @Field(() => FiltersMaxOrderByAggregateInput, {nullable:true})
    _max?: FiltersMaxOrderByAggregateInput;

    @Field(() => FiltersMinOrderByAggregateInput, {nullable:true})
    _min?: FiltersMinOrderByAggregateInput;

    @Field(() => FiltersSumOrderByAggregateInput, {nullable:true})
    _sum?: FiltersSumOrderByAggregateInput;
}
