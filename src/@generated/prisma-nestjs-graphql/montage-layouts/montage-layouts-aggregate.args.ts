import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereInput } from './montage-layouts-where.input';
import { Type } from 'class-transformer';
import { MontageLayoutsOrderByWithRelationInput } from './montage-layouts-order-by-with-relation.input';
import { MontageLayoutsWhereUniqueInput } from './montage-layouts-where-unique.input';
import { Int } from '@nestjs/graphql';
import { MontageLayoutsCountAggregateInput } from './montage-layouts-count-aggregate.input';
import { MontageLayoutsAvgAggregateInput } from './montage-layouts-avg-aggregate.input';
import { MontageLayoutsSumAggregateInput } from './montage-layouts-sum-aggregate.input';
import { MontageLayoutsMinAggregateInput } from './montage-layouts-min-aggregate.input';
import { MontageLayoutsMaxAggregateInput } from './montage-layouts-max-aggregate.input';

@ArgsType()
export class MontageLayoutsAggregateArgs {

    @Field(() => MontageLayoutsWhereInput, {nullable:true})
    @Type(() => MontageLayoutsWhereInput)
    where?: MontageLayoutsWhereInput;

    @Field(() => [MontageLayoutsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<MontageLayoutsOrderByWithRelationInput>;

    @Field(() => MontageLayoutsWhereUniqueInput, {nullable:true})
    cursor?: MontageLayoutsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => MontageLayoutsCountAggregateInput, {nullable:true})
    _count?: MontageLayoutsCountAggregateInput;

    @Field(() => MontageLayoutsAvgAggregateInput, {nullable:true})
    _avg?: MontageLayoutsAvgAggregateInput;

    @Field(() => MontageLayoutsSumAggregateInput, {nullable:true})
    _sum?: MontageLayoutsSumAggregateInput;

    @Field(() => MontageLayoutsMinAggregateInput, {nullable:true})
    _min?: MontageLayoutsMinAggregateInput;

    @Field(() => MontageLayoutsMaxAggregateInput, {nullable:true})
    _max?: MontageLayoutsMaxAggregateInput;
}
