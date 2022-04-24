import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { ConfigCountOrderByAggregateInput } from './config-count-order-by-aggregate.input';
import { ConfigAvgOrderByAggregateInput } from './config-avg-order-by-aggregate.input';
import { ConfigMaxOrderByAggregateInput } from './config-max-order-by-aggregate.input';
import { ConfigMinOrderByAggregateInput } from './config-min-order-by-aggregate.input';
import { ConfigSumOrderByAggregateInput } from './config-sum-order-by-aggregate.input';

@InputType()
export class ConfigOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Value?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultValue?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Hint?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Pattern?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Format?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Prompt?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Help?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Category?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Readonly?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Requires?: keyof typeof SortOrder;

    @Field(() => ConfigCountOrderByAggregateInput, {nullable:true})
    _count?: ConfigCountOrderByAggregateInput;

    @Field(() => ConfigAvgOrderByAggregateInput, {nullable:true})
    _avg?: ConfigAvgOrderByAggregateInput;

    @Field(() => ConfigMaxOrderByAggregateInput, {nullable:true})
    _max?: ConfigMaxOrderByAggregateInput;

    @Field(() => ConfigMinOrderByAggregateInput, {nullable:true})
    _min?: ConfigMinOrderByAggregateInput;

    @Field(() => ConfigSumOrderByAggregateInput, {nullable:true})
    _sum?: ConfigSumOrderByAggregateInput;
}
