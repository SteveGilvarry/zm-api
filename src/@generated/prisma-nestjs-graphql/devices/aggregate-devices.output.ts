import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { DevicesCountAggregate } from './devices-count-aggregate.output';
import { DevicesAvgAggregate } from './devices-avg-aggregate.output';
import { DevicesSumAggregate } from './devices-sum-aggregate.output';
import { DevicesMinAggregate } from './devices-min-aggregate.output';
import { DevicesMaxAggregate } from './devices-max-aggregate.output';

@ObjectType()
export class AggregateDevices {

    @Field(() => DevicesCountAggregate, {nullable:true})
    _count?: DevicesCountAggregate;

    @Field(() => DevicesAvgAggregate, {nullable:true})
    _avg?: DevicesAvgAggregate;

    @Field(() => DevicesSumAggregate, {nullable:true})
    _sum?: DevicesSumAggregate;

    @Field(() => DevicesMinAggregate, {nullable:true})
    _min?: DevicesMinAggregate;

    @Field(() => DevicesMaxAggregate, {nullable:true})
    _max?: DevicesMaxAggregate;
}
