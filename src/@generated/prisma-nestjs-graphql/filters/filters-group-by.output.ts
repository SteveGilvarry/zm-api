import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { FiltersCountAggregate } from './filters-count-aggregate.output';
import { FiltersAvgAggregate } from './filters-avg-aggregate.output';
import { FiltersSumAggregate } from './filters-sum-aggregate.output';
import { FiltersMinAggregate } from './filters-min-aggregate.output';
import { FiltersMaxAggregate } from './filters-max-aggregate.output';

@ObjectType()
export class FiltersGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Int, {nullable:true})
    UserId?: number;

    @Field(() => String, {nullable:false})
    Query_json!: string;

    @Field(() => Int, {nullable:false})
    AutoArchive!: number;

    @Field(() => Int, {nullable:false})
    AutoUnarchive!: number;

    @Field(() => Int, {nullable:false})
    AutoVideo!: number;

    @Field(() => Int, {nullable:false})
    AutoUpload!: number;

    @Field(() => Int, {nullable:false})
    AutoEmail!: number;

    @Field(() => String, {nullable:true})
    EmailTo?: string;

    @Field(() => String, {nullable:true})
    EmailSubject?: string;

    @Field(() => String, {nullable:true})
    EmailBody?: string;

    @Field(() => Int, {nullable:false})
    AutoMessage!: number;

    @Field(() => Int, {nullable:false})
    AutoExecute!: number;

    @Field(() => String, {nullable:true})
    AutoExecuteCmd?: string;

    @Field(() => Int, {nullable:false})
    AutoDelete!: number;

    @Field(() => Int, {nullable:false})
    AutoMove!: number;

    @Field(() => Int, {nullable:false})
    AutoCopy!: number;

    @Field(() => Int, {nullable:false})
    AutoCopyTo!: number;

    @Field(() => Int, {nullable:false})
    AutoMoveTo!: number;

    @Field(() => Int, {nullable:false})
    UpdateDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    Background!: number;

    @Field(() => Int, {nullable:false})
    Concurrent!: number;

    @Field(() => Int, {nullable:false})
    LockRows!: number;

    @Field(() => FiltersCountAggregate, {nullable:true})
    _count?: FiltersCountAggregate;

    @Field(() => FiltersAvgAggregate, {nullable:true})
    _avg?: FiltersAvgAggregate;

    @Field(() => FiltersSumAggregate, {nullable:true})
    _sum?: FiltersSumAggregate;

    @Field(() => FiltersMinAggregate, {nullable:true})
    _min?: FiltersMinAggregate;

    @Field(() => FiltersMaxAggregate, {nullable:true})
    _max?: FiltersMaxAggregate;
}
