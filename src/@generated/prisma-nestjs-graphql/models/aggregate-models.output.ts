import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ModelsCountAggregate } from './models-count-aggregate.output';
import { ModelsAvgAggregate } from './models-avg-aggregate.output';
import { ModelsSumAggregate } from './models-sum-aggregate.output';
import { ModelsMinAggregate } from './models-min-aggregate.output';
import { ModelsMaxAggregate } from './models-max-aggregate.output';

@ObjectType()
export class AggregateModels {

    @Field(() => ModelsCountAggregate, {nullable:true})
    _count?: ModelsCountAggregate;

    @Field(() => ModelsAvgAggregate, {nullable:true})
    _avg?: ModelsAvgAggregate;

    @Field(() => ModelsSumAggregate, {nullable:true})
    _sum?: ModelsSumAggregate;

    @Field(() => ModelsMinAggregate, {nullable:true})
    _min?: ModelsMinAggregate;

    @Field(() => ModelsMaxAggregate, {nullable:true})
    _max?: ModelsMaxAggregate;
}
