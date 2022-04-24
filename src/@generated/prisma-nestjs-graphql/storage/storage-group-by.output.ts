import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Storage_Type } from '../prisma/storage-type.enum';
import { Storage_Scheme } from '../prisma/storage-scheme.enum';
import { StorageCountAggregate } from './storage-count-aggregate.output';
import { StorageAvgAggregate } from './storage-avg-aggregate.output';
import { StorageSumAggregate } from './storage-sum-aggregate.output';
import { StorageMinAggregate } from './storage-min-aggregate.output';
import { StorageMaxAggregate } from './storage-max-aggregate.output';

@ObjectType()
export class StorageGroupBy {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false})
    Path!: string;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => Storage_Type, {nullable:false})
    Type!: keyof typeof Storage_Type;

    @Field(() => String, {nullable:true})
    Url?: string;

    @Field(() => String, {nullable:true})
    DiskSpace?: bigint | number;

    @Field(() => Storage_Scheme, {nullable:false})
    Scheme!: keyof typeof Storage_Scheme;

    @Field(() => Int, {nullable:true})
    ServerId?: number;

    @Field(() => Boolean, {nullable:false})
    DoDelete!: boolean;

    @Field(() => Boolean, {nullable:false})
    Enabled!: boolean;

    @Field(() => StorageCountAggregate, {nullable:true})
    _count?: StorageCountAggregate;

    @Field(() => StorageAvgAggregate, {nullable:true})
    _avg?: StorageAvgAggregate;

    @Field(() => StorageSumAggregate, {nullable:true})
    _sum?: StorageSumAggregate;

    @Field(() => StorageMinAggregate, {nullable:true})
    _min?: StorageMinAggregate;

    @Field(() => StorageMaxAggregate, {nullable:true})
    _max?: StorageMaxAggregate;
}
