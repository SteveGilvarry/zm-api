import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereInput } from './controls-where.input';
import { Type } from 'class-transformer';
import { ControlsOrderByWithAggregationInput } from './controls-order-by-with-aggregation.input';
import { ControlsScalarFieldEnum } from './controls-scalar-field.enum';
import { ControlsScalarWhereWithAggregatesInput } from './controls-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { ControlsCountAggregateInput } from './controls-count-aggregate.input';
import { ControlsAvgAggregateInput } from './controls-avg-aggregate.input';
import { ControlsSumAggregateInput } from './controls-sum-aggregate.input';
import { ControlsMinAggregateInput } from './controls-min-aggregate.input';
import { ControlsMaxAggregateInput } from './controls-max-aggregate.input';

@ArgsType()
export class ControlsGroupByArgs {

    @Field(() => ControlsWhereInput, {nullable:true})
    @Type(() => ControlsWhereInput)
    where?: ControlsWhereInput;

    @Field(() => [ControlsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<ControlsOrderByWithAggregationInput>;

    @Field(() => [ControlsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof ControlsScalarFieldEnum>;

    @Field(() => ControlsScalarWhereWithAggregatesInput, {nullable:true})
    having?: ControlsScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ControlsCountAggregateInput, {nullable:true})
    _count?: ControlsCountAggregateInput;

    @Field(() => ControlsAvgAggregateInput, {nullable:true})
    _avg?: ControlsAvgAggregateInput;

    @Field(() => ControlsSumAggregateInput, {nullable:true})
    _sum?: ControlsSumAggregateInput;

    @Field(() => ControlsMinAggregateInput, {nullable:true})
    _min?: ControlsMinAggregateInput;

    @Field(() => ControlsMaxAggregateInput, {nullable:true})
    _max?: ControlsMaxAggregateInput;
}
