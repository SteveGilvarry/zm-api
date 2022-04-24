import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ZonePresetsCountAggregate } from './zone-presets-count-aggregate.output';
import { ZonePresetsAvgAggregate } from './zone-presets-avg-aggregate.output';
import { ZonePresetsSumAggregate } from './zone-presets-sum-aggregate.output';
import { ZonePresetsMinAggregate } from './zone-presets-min-aggregate.output';
import { ZonePresetsMaxAggregate } from './zone-presets-max-aggregate.output';

@ObjectType()
export class AggregateZonePresets {

    @Field(() => ZonePresetsCountAggregate, {nullable:true})
    _count?: ZonePresetsCountAggregate;

    @Field(() => ZonePresetsAvgAggregate, {nullable:true})
    _avg?: ZonePresetsAvgAggregate;

    @Field(() => ZonePresetsSumAggregate, {nullable:true})
    _sum?: ZonePresetsSumAggregate;

    @Field(() => ZonePresetsMinAggregate, {nullable:true})
    _min?: ZonePresetsMinAggregate;

    @Field(() => ZonePresetsMaxAggregate, {nullable:true})
    _max?: ZonePresetsMaxAggregate;
}
