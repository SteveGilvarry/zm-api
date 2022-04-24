import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class DevicesCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Type!: number;

    @Field(() => Int, {nullable:false})
    KeyString!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
