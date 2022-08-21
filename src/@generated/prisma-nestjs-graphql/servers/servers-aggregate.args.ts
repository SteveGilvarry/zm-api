import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereInput } from './servers-where.input';
import { Type } from 'class-transformer';
import { ServersOrderByWithRelationInput } from './servers-order-by-with-relation.input';
import { ServersWhereUniqueInput } from './servers-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ServersCountAggregateInput } from './servers-count-aggregate.input';
import { ServersAvgAggregateInput } from './servers-avg-aggregate.input';
import { ServersSumAggregateInput } from './servers-sum-aggregate.input';
import { ServersMinAggregateInput } from './servers-min-aggregate.input';
import { ServersMaxAggregateInput } from './servers-max-aggregate.input';

@ArgsType()
export class ServersAggregateArgs {

    @Field(() => ServersWhereInput, {nullable:true})
    @Type(() => ServersWhereInput)
    where?: ServersWhereInput;

    @Field(() => [ServersOrderByWithRelationInput], {nullable:true})
    @Type(() => ServersOrderByWithRelationInput)
    orderBy?: Array<ServersOrderByWithRelationInput>;

    @Field(() => ServersWhereUniqueInput, {nullable:true})
    @Type(() => ServersWhereUniqueInput)
    cursor?: ServersWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ServersCountAggregateInput, {nullable:true})
    @Type(() => ServersCountAggregateInput)
    _count?: ServersCountAggregateInput;

    @Field(() => ServersAvgAggregateInput, {nullable:true})
    @Type(() => ServersAvgAggregateInput)
    _avg?: ServersAvgAggregateInput;

    @Field(() => ServersSumAggregateInput, {nullable:true})
    @Type(() => ServersSumAggregateInput)
    _sum?: ServersSumAggregateInput;

    @Field(() => ServersMinAggregateInput, {nullable:true})
    @Type(() => ServersMinAggregateInput)
    _min?: ServersMinAggregateInput;

    @Field(() => ServersMaxAggregateInput, {nullable:true})
    @Type(() => ServersMaxAggregateInput)
    _max?: ServersMaxAggregateInput;
}
