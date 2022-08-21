import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereInput } from './montage-layouts-where.input';
import { Type } from 'class-transformer';
import { MontageLayoutsOrderByWithAggregationInput } from './montage-layouts-order-by-with-aggregation.input';
import { MontageLayoutsScalarFieldEnum } from './montage-layouts-scalar-field.enum';
import { MontageLayoutsScalarWhereWithAggregatesInput } from './montage-layouts-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { MontageLayoutsCountAggregateInput } from './montage-layouts-count-aggregate.input';
import { MontageLayoutsAvgAggregateInput } from './montage-layouts-avg-aggregate.input';
import { MontageLayoutsSumAggregateInput } from './montage-layouts-sum-aggregate.input';
import { MontageLayoutsMinAggregateInput } from './montage-layouts-min-aggregate.input';
import { MontageLayoutsMaxAggregateInput } from './montage-layouts-max-aggregate.input';

@ArgsType()
export class MontageLayoutsGroupByArgs {

    @Field(() => MontageLayoutsWhereInput, {nullable:true})
    @Type(() => MontageLayoutsWhereInput)
    where?: MontageLayoutsWhereInput;

    @Field(() => [MontageLayoutsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<MontageLayoutsOrderByWithAggregationInput>;

    @Field(() => [MontageLayoutsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof MontageLayoutsScalarFieldEnum>;

    @Field(() => MontageLayoutsScalarWhereWithAggregatesInput, {nullable:true})
    having?: MontageLayoutsScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => MontageLayoutsCountAggregateInput, {nullable:true})
    _count?: MontageLayoutsCountAggregateInput;

    @Field(() => MontageLayoutsAvgAggregateInput, {nullable:true})
    _avg?: MontageLayoutsAvgAggregateInput;

    @Field(() => MontageLayoutsSumAggregateInput, {nullable:true})
    _sum?: MontageLayoutsSumAggregateInput;

    @Field(() => MontageLayoutsMinAggregateInput, {nullable:true})
    _min?: MontageLayoutsMinAggregateInput;

    @Field(() => MontageLayoutsMaxAggregateInput, {nullable:true})
    _max?: MontageLayoutsMaxAggregateInput;
}
