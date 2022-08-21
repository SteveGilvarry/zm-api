import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereInput } from './control-presets-where.input';
import { Type } from 'class-transformer';
import { ControlPresetsOrderByWithAggregationInput } from './control-presets-order-by-with-aggregation.input';
import { ControlPresetsScalarFieldEnum } from './control-presets-scalar-field.enum';
import { ControlPresetsScalarWhereWithAggregatesInput } from './control-presets-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { ControlPresetsCountAggregateInput } from './control-presets-count-aggregate.input';
import { ControlPresetsAvgAggregateInput } from './control-presets-avg-aggregate.input';
import { ControlPresetsSumAggregateInput } from './control-presets-sum-aggregate.input';
import { ControlPresetsMinAggregateInput } from './control-presets-min-aggregate.input';
import { ControlPresetsMaxAggregateInput } from './control-presets-max-aggregate.input';

@ArgsType()
export class ControlPresetsGroupByArgs {

    @Field(() => ControlPresetsWhereInput, {nullable:true})
    @Type(() => ControlPresetsWhereInput)
    where?: ControlPresetsWhereInput;

    @Field(() => [ControlPresetsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<ControlPresetsOrderByWithAggregationInput>;

    @Field(() => [ControlPresetsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof ControlPresetsScalarFieldEnum>;

    @Field(() => ControlPresetsScalarWhereWithAggregatesInput, {nullable:true})
    having?: ControlPresetsScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ControlPresetsCountAggregateInput, {nullable:true})
    _count?: ControlPresetsCountAggregateInput;

    @Field(() => ControlPresetsAvgAggregateInput, {nullable:true})
    _avg?: ControlPresetsAvgAggregateInput;

    @Field(() => ControlPresetsSumAggregateInput, {nullable:true})
    _sum?: ControlPresetsSumAggregateInput;

    @Field(() => ControlPresetsMinAggregateInput, {nullable:true})
    _min?: ControlPresetsMinAggregateInput;

    @Field(() => ControlPresetsMaxAggregateInput, {nullable:true})
    _max?: ControlPresetsMaxAggregateInput;
}
