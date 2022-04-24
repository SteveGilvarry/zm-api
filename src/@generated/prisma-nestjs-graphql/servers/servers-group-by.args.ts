import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereInput } from './servers-where.input';
import { ServersOrderByWithAggregationInput } from './servers-order-by-with-aggregation.input';
import { ServersScalarFieldEnum } from './servers-scalar-field.enum';
import { ServersScalarWhereWithAggregatesInput } from './servers-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { ServersCountAggregateInput } from './servers-count-aggregate.input';
import { ServersAvgAggregateInput } from './servers-avg-aggregate.input';
import { ServersSumAggregateInput } from './servers-sum-aggregate.input';
import { ServersMinAggregateInput } from './servers-min-aggregate.input';
import { ServersMaxAggregateInput } from './servers-max-aggregate.input';

@ArgsType()
export class ServersGroupByArgs {

    @Field(() => ServersWhereInput, {nullable:true})
    where?: ServersWhereInput;

    @Field(() => [ServersOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<ServersOrderByWithAggregationInput>;

    @Field(() => [ServersScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof ServersScalarFieldEnum>;

    @Field(() => ServersScalarWhereWithAggregatesInput, {nullable:true})
    having?: ServersScalarWhereWithAggregatesInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ServersCountAggregateInput, {nullable:true})
    _count?: ServersCountAggregateInput;

    @Field(() => ServersAvgAggregateInput, {nullable:true})
    _avg?: ServersAvgAggregateInput;

    @Field(() => ServersSumAggregateInput, {nullable:true})
    _sum?: ServersSumAggregateInput;

    @Field(() => ServersMinAggregateInput, {nullable:true})
    _min?: ServersMinAggregateInput;

    @Field(() => ServersMaxAggregateInput, {nullable:true})
    _max?: ServersMaxAggregateInput;
}
