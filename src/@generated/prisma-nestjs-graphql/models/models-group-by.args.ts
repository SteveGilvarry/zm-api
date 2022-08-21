import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ModelsWhereInput } from './models-where.input';
import { Type } from 'class-transformer';
import { ModelsOrderByWithAggregationInput } from './models-order-by-with-aggregation.input';
import { ModelsScalarFieldEnum } from './models-scalar-field.enum';
import { ModelsScalarWhereWithAggregatesInput } from './models-scalar-where-with-aggregates.input';
import { Int } from '@nestjs/graphql';
import { ModelsCountAggregateInput } from './models-count-aggregate.input';
import { ModelsAvgAggregateInput } from './models-avg-aggregate.input';
import { ModelsSumAggregateInput } from './models-sum-aggregate.input';
import { ModelsMinAggregateInput } from './models-min-aggregate.input';
import { ModelsMaxAggregateInput } from './models-max-aggregate.input';

@ArgsType()
export class ModelsGroupByArgs {

    @Field(() => ModelsWhereInput, {nullable:true})
    @Type(() => ModelsWhereInput)
    where?: ModelsWhereInput;

    @Field(() => [ModelsOrderByWithAggregationInput], {nullable:true})
    orderBy?: Array<ModelsOrderByWithAggregationInput>;

    @Field(() => [ModelsScalarFieldEnum], {nullable:false})
    by!: Array<keyof typeof ModelsScalarFieldEnum>;

    @Field(() => ModelsScalarWhereWithAggregatesInput, {nullable:true})
    having?: ModelsScalarWhereWithAggregatesInput;

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
