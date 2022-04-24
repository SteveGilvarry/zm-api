import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class SnapshotsCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Description!: number;

    @Field(() => Int, {nullable:false})
    CreatedBy!: number;

    @Field(() => Int, {nullable:false})
    CreatedOn!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
