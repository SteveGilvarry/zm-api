import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ControlsCountAggregate } from './controls-count-aggregate.output';
import { ControlsAvgAggregate } from './controls-avg-aggregate.output';
import { ControlsSumAggregate } from './controls-sum-aggregate.output';
import { ControlsMinAggregate } from './controls-min-aggregate.output';
import { ControlsMaxAggregate } from './controls-max-aggregate.output';

@ObjectType()
export class AggregateControls {

    @Field(() => ControlsCountAggregate, {nullable:true})
    _count?: ControlsCountAggregate;

    @Field(() => ControlsAvgAggregate, {nullable:true})
    _avg?: ControlsAvgAggregate;

    @Field(() => ControlsSumAggregate, {nullable:true})
    _sum?: ControlsSumAggregate;

    @Field(() => ControlsMinAggregate, {nullable:true})
    _min?: ControlsMinAggregate;

    @Field(() => ControlsMaxAggregate, {nullable:true})
    _max?: ControlsMaxAggregate;
}
