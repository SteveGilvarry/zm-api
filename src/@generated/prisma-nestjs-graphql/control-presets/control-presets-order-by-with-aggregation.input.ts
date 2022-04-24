import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { ControlPresetsCountOrderByAggregateInput } from './control-presets-count-order-by-aggregate.input';
import { ControlPresetsAvgOrderByAggregateInput } from './control-presets-avg-order-by-aggregate.input';
import { ControlPresetsMaxOrderByAggregateInput } from './control-presets-max-order-by-aggregate.input';
import { ControlPresetsMinOrderByAggregateInput } from './control-presets-min-order-by-aggregate.input';
import { ControlPresetsSumOrderByAggregateInput } from './control-presets-sum-order-by-aggregate.input';

@InputType()
export class ControlPresetsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Preset?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Label?: keyof typeof SortOrder;

    @Field(() => ControlPresetsCountOrderByAggregateInput, {nullable:true})
    _count?: ControlPresetsCountOrderByAggregateInput;

    @Field(() => ControlPresetsAvgOrderByAggregateInput, {nullable:true})
    _avg?: ControlPresetsAvgOrderByAggregateInput;

    @Field(() => ControlPresetsMaxOrderByAggregateInput, {nullable:true})
    _max?: ControlPresetsMaxOrderByAggregateInput;

    @Field(() => ControlPresetsMinOrderByAggregateInput, {nullable:true})
    _min?: ControlPresetsMinOrderByAggregateInput;

    @Field(() => ControlPresetsSumOrderByAggregateInput, {nullable:true})
    _sum?: ControlPresetsSumOrderByAggregateInput;
}
