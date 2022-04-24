import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereInput } from './devices-where.input';
import { DevicesOrderByWithAggregationInput } from './devices-order-by-with-aggregation.input';
import { DevicesScalarFieldEnum } from './devices-scalar-field.enum';
import { DevicesScalarWhereWithAggregatesInput } from './devices-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { DevicesCountAggregateInput } from './devices-count-aggregate.input';
import { DevicesAvgAggregateInput } from './devices-avg-aggregate.input';
import { DevicesSumAggregateInput } from './devices-sum-aggregate.input';
import { DevicesMinAggregateInput } from './devices-min-aggregate.input';
import { DevicesMaxAggregateInput } from './devices-max-aggregate.input';

@ArgsType()
export class DevicesGroupByArgs {

    @Field(() => DevicesWhereInput, {nullable:true})
    where?: DevicesWhereInput;

    @Field(() => [DevicesOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<DevicesOrderByWithAggregationInput>;

    @Field(() => [DevicesScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof DevicesScalarFieldEnum>;

    @Field(() => DevicesScalarWhereWithAggregatesInput, {nullable:true})
    having?: DevicesScalarWhereWithAggregatesInput;

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
