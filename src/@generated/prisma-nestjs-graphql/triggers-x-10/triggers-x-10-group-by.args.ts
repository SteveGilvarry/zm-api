import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10WhereInput } from './triggers-x-10-where.input';
import { TriggersX10OrderByWithAggregationInput } from './triggers-x-10-order-by-with-aggregation.input';
import { TriggersX10ScalarFieldEnum } from './triggers-x-10-scalar-field.enum';
import { TriggersX10ScalarWhereWithAggregatesInput } from './triggers-x-10-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { TriggersX10CountAggregateInput } from './triggers-x-10-count-aggregate.input';
import { TriggersX10AvgAggregateInput } from './triggers-x-10-avg-aggregate.input';
import { TriggersX10SumAggregateInput } from './triggers-x-10-sum-aggregate.input';
import { TriggersX10MinAggregateInput } from './triggers-x-10-min-aggregate.input';
import { TriggersX10MaxAggregateInput } from './triggers-x-10-max-aggregate.input';

@ArgsType()
export class TriggersX10GroupByArgs {

    @Field(() => TriggersX10WhereInput, {nullable:true})
    where?: TriggersX10WhereInput;

    @Field(() => [TriggersX10OrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<TriggersX10OrderByWithAggregationInput>;

    @Field(() => [TriggersX10ScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof TriggersX10ScalarFieldEnum>;

    @Field(() => TriggersX10ScalarWhereWithAggregatesInput, {nullable:true})
    having?: TriggersX10ScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => TriggersX10CountAggregateInput, {nullable:true})
    _count?: TriggersX10CountAggregateInput;

    @Field(() => TriggersX10AvgAggregateInput, {nullable:true})
    _avg?: TriggersX10AvgAggregateInput;

    @Field(() => TriggersX10SumAggregateInput, {nullable:true})
    _sum?: TriggersX10SumAggregateInput;

    @Field(() => TriggersX10MinAggregateInput, {nullable:true})
    _min?: TriggersX10MinAggregateInput;

    @Field(() => TriggersX10MaxAggregateInput, {nullable:true})
    _max?: TriggersX10MaxAggregateInput;
}
