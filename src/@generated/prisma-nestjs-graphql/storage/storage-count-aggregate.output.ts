import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class StorageCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Path!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Type!: number;

    @Field(() => Int, {nullable:false})
    Url!: number;

    @Field(() => Int, {nullable:false})
    DiskSpace!: number;

    @Field(() => Int, {nullable:false})
    Scheme!: number;

    @Field(() => Int, {nullable:false})
    ServerId!: number;

    @Field(() => Int, {nullable:false})
    DoDelete!: number;

    @Field(() => Int, {nullable:false})
    Enabled!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
