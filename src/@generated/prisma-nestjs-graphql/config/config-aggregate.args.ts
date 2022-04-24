import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigWhereInput } from './config-where.input';
import { ConfigOrderByWithRelationInput } from './config-order-by-with-relation.input';
import { ConfigWhereUniqueInput } from './config-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ConfigCountAggregateInput } from './config-count-aggregate.input';
import { ConfigAvgAggregateInput } from './config-avg-aggregate.input';
import { ConfigSumAggregateInput } from './config-sum-aggregate.input';
import { ConfigMinAggregateInput } from './config-min-aggregate.input';
import { ConfigMaxAggregateInput } from './config-max-aggregate.input';

@ArgsType()
export class ConfigAggregateArgs {

    @Field(() => ConfigWhereInput, {nullable:true})
    where?: ConfigWhereInput;

    @Field(() => [ConfigOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ConfigOrderByWithRelationInput>;

    @Field(() => ConfigWhereUniqueInput, {nullable:true})
    cursor?: ConfigWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ConfigCountAggregateInput, {nullable:true})
    _count?: ConfigCountAggregateInput;

    @Field(() => ConfigAvgAggregateInput, {nullable:true})
    _avg?: ConfigAvgAggregateInput;

    @Field(() => ConfigSumAggregateInput, {nullable:true})
    _sum?: ConfigSumAggregateInput;

    @Field(() => ConfigMinAggregateInput, {nullable:true})
    _min?: ConfigMinAggregateInput;

    @Field(() => ConfigMaxAggregateInput, {nullable:true})
    _max?: ConfigMaxAggregateInput;
}
