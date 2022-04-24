import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereInput } from './models-where.input';
import { ModelsOrderByWithRelationInput } from './models-order-by-with-relation.input';
import { ModelsWhereUniqueInput } from './models-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ModelsCountAggregateInput } from './models-count-aggregate.input';
import { ModelsAvgAggregateInput } from './models-avg-aggregate.input';
import { ModelsSumAggregateInput } from './models-sum-aggregate.input';
import { ModelsMinAggregateInput } from './models-min-aggregate.input';
import { ModelsMaxAggregateInput } from './models-max-aggregate.input';

@ArgsType()
export class ModelsAggregateArgs {

    @Field(() => ModelsWhereInput, {nullable:true})
    where?: ModelsWhereInput;

    @Field(() => [ModelsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ModelsOrderByWithRelationInput>;

    @Field(() => ModelsWhereUniqueInput, {nullable:true})
    cursor?: ModelsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => ModelsCountAggregateInput, {nullable:true})
    _count?: ModelsCountAggregateInput;

    @Field(() => ModelsAvgAggregateInput, {nullable:true})
    _avg?: ModelsAvgAggregateInput;

    @Field(() => ModelsSumAggregateInput, {nullable:true})
    _sum?: ModelsSumAggregateInput;

    @Field(() => ModelsMinAggregateInput, {nullable:true})
    _min?: ModelsMinAggregateInput;

    @Field(() => ModelsMaxAggregateInput, {nullable:true})
    _max?: ModelsMaxAggregateInput;
}
