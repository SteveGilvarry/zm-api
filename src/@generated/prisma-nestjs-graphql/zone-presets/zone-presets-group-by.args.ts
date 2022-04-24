import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereInput } from './zone-presets-where.input';
import { ZonePresetsOrderByWithAggregationInput } from './zone-presets-order-by-with-aggregation.input';
import { ZonePresetsScalarFieldEnum } from './zone-presets-scalar-field.enum';
import { ZonePresetsScalarWhereWithAggregatesInput } from './zone-presets-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { ZonePresetsCountAggregateInput } from './zone-presets-count-aggregate.input';
import { ZonePresetsAvgAggregateInput } from './zone-presets-avg-aggregate.input';
import { ZonePresetsSumAggregateInput } from './zone-presets-sum-aggregate.input';
import { ZonePresetsMinAggregateInput } from './zone-presets-min-aggregate.input';
import { ZonePresetsMaxAggregateInput } from './zone-presets-max-aggregate.input';

@ArgsType()
export class ZonePresetsGroupByArgs {

    @Field(() => ZonePresetsWhereInput, {nullable:true})
    where?: ZonePresetsWhereInput;

    @Field(() => [ZonePresetsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<ZonePresetsOrderByWithAggregationInput>;

    @Field(() => [ZonePresetsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof ZonePresetsScalarFieldEnum>;

    @Field(() => ZonePresetsScalarWhereWithAggregatesInput, {nullable:true})
    having?: ZonePresetsScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ZonePresetsCountAggregateInput, {nullable:true})
    _count?: ZonePresetsCountAggregateInput;

    @Field(() => ZonePresetsAvgAggregateInput, {nullable:true})
    _avg?: ZonePresetsAvgAggregateInput;

    @Field(() => ZonePresetsSumAggregateInput, {nullable:true})
    _sum?: ZonePresetsSumAggregateInput;

    @Field(() => ZonePresetsMinAggregateInput, {nullable:true})
    _min?: ZonePresetsMinAggregateInput;

    @Field(() => ZonePresetsMaxAggregateInput, {nullable:true})
    _max?: ZonePresetsMaxAggregateInput;
}
