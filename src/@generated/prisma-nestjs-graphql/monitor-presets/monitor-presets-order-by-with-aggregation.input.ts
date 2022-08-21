import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { MonitorPresetsCountOrderByAggregateInput } from './monitor-presets-count-order-by-aggregate.input';
import { Type } from 'class-transformer';
import { MonitorPresetsAvgOrderByAggregateInput } from './monitor-presets-avg-order-by-aggregate.input';
import { MonitorPresetsMaxOrderByAggregateInput } from './monitor-presets-max-order-by-aggregate.input';
import { MonitorPresetsMinOrderByAggregateInput } from './monitor-presets-min-order-by-aggregate.input';
import { MonitorPresetsSumOrderByAggregateInput } from './monitor-presets-sum-order-by-aggregate.input';

@InputType()
export class MonitorPresetsOrderByWithAggregationInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Device?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Channel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Format?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Protocol?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Method?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Host?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Port?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Path?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SubPath?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Width?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Height?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Palette?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxFPS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Controllable?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ControlId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ControlDevice?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ControlAddress?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultRate?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultScale?: keyof typeof SortOrder;

    @Field(() => MonitorPresetsCountOrderByAggregateInput, {nullable:true})
    @Type(() => MonitorPresetsCountOrderByAggregateInput)
    _count?: MonitorPresetsCountOrderByAggregateInput;

    @Field(() => MonitorPresetsAvgOrderByAggregateInput, {nullable:true})
    @Type(() => MonitorPresetsAvgOrderByAggregateInput)
    _avg?: MonitorPresetsAvgOrderByAggregateInput;

    @Field(() => MonitorPresetsMaxOrderByAggregateInput, {nullable:true})
    @Type(() => MonitorPresetsMaxOrderByAggregateInput)
    _max?: MonitorPresetsMaxOrderByAggregateInput;

    @Field(() => MonitorPresetsMinOrderByAggregateInput, {nullable:true})
    @Type(() => MonitorPresetsMinOrderByAggregateInput)
    _min?: MonitorPresetsMinOrderByAggregateInput;

    @Field(() => MonitorPresetsSumOrderByAggregateInput, {nullable:true})
    @Type(() => MonitorPresetsSumOrderByAggregateInput)
    _sum?: MonitorPresetsSumOrderByAggregateInput;
}
