import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereInput } from './control-presets-where.input';
import { ControlPresetsOrderByWithRelationInput } from './control-presets-order-by-with-relation.input';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ControlPresetsCountAggregateInput } from './control-presets-count-aggregate.input';
import { ControlPresetsAvgAggregateInput } from './control-presets-avg-aggregate.input';
import { ControlPresetsSumAggregateInput } from './control-presets-sum-aggregate.input';
import { ControlPresetsMinAggregateInput } from './control-presets-min-aggregate.input';
import { ControlPresetsMaxAggregateInput } from './control-presets-max-aggregate.input';

@ArgsType()
export class ControlPresetsAggregateArgs {

    @Field(() => ControlPresetsWhereInput, {nullable:true})
    where?: ControlPresetsWhereInput;

    @Field(() => [ControlPresetsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ControlPresetsOrderByWithRelationInput>;

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:true})
    cursor?: ControlPresetsWhereUniqueInput;

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
