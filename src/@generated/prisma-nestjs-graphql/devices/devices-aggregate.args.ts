import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereInput } from './devices-where.input';
import { DevicesOrderByWithRelationInput } from './devices-order-by-with-relation.input';
import { DevicesWhereUniqueInput } from './devices-where-unique.input';
import { Int } from '@nestjs/graphql';
import { DevicesCountAggregateInput } from './devices-count-aggregate.input';
import { DevicesAvgAggregateInput } from './devices-avg-aggregate.input';
import { DevicesSumAggregateInput } from './devices-sum-aggregate.input';
import { DevicesMinAggregateInput } from './devices-min-aggregate.input';
import { DevicesMaxAggregateInput } from './devices-max-aggregate.input';

@ArgsType()
export class DevicesAggregateArgs {

    @Field(() => DevicesWhereInput, {nullable:true})
    where?: DevicesWhereInput;

    @Field(() => [DevicesOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<DevicesOrderByWithRelationInput>;

    @Field(() => DevicesWhereUniqueInput, {nullable:true})
    cursor?: DevicesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => DevicesCountAggregateInput, {nullable:true})
    _count?: DevicesCountAggregateInput;

    @Field(() => DevicesAvgAggregateInput, {nullable:true})
    _avg?: DevicesAvgAggregateInput;

    @Field(() => DevicesSumAggregateInput, {nullable:true})
    _sum?: DevicesSumAggregateInput;

    @Field(() => DevicesMinAggregateInput, {nullable:true})
    _min?: DevicesMinAggregateInput;

    @Field(() => DevicesMaxAggregateInput, {nullable:true})
    _max?: DevicesMaxAggregateInput;
}
