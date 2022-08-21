import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereInput } from './zone-presets-where.input';
import { Type } from 'class-transformer';
import { ZonePresetsOrderByWithRelationInput } from './zone-presets-order-by-with-relation.input';
import { ZonePresetsWhereUniqueInput } from './zone-presets-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ZonePresetsCountAggregateInput } from './zone-presets-count-aggregate.input';
import { ZonePresetsAvgAggregateInput } from './zone-presets-avg-aggregate.input';
import { ZonePresetsSumAggregateInput } from './zone-presets-sum-aggregate.input';
import { ZonePresetsMinAggregateInput } from './zone-presets-min-aggregate.input';
import { ZonePresetsMaxAggregateInput } from './zone-presets-max-aggregate.input';

@ArgsType()
export class ZonePresetsAggregateArgs {

    @Field(() => ZonePresetsWhereInput, {nullable:true})
    @Type(() => ZonePresetsWhereInput)
    where?: ZonePresetsWhereInput;

    @Field(() => [ZonePresetsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ZonePresetsOrderByWithRelationInput>;

    @Field(() => ZonePresetsWhereUniqueInput, {nullable:true})
    cursor?: ZonePresetsWhereUniqueInput;

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
