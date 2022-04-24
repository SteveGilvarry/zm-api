import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { ModelsCountAggregate } from './models-count-aggregate.output';
import { ModelsAvgAggregate } from './models-avg-aggregate.output';
import { ModelsSumAggregate } from './models-sum-aggregate.output';
import { ModelsMinAggregate } from './models-min-aggregate.output';
import { ModelsMaxAggregate } from './models-max-aggregate.output';

@ObjectType()
export class ModelsGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Int, {nullable:true})
    ManufacturerId?: number;

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
