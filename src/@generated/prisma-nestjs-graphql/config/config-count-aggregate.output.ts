import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ConfigCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    Value!: number;

    @Field(() => Int, {nullable:false})
    Type!: number;

    @Field(() => Int, {nullable:false})
    DefaultValue!: number;

    @Field(() => Int, {nullable:false})
    Hint!: number;

    @Field(() => Int, {nullable:false})
    Pattern!: number;

    @Field(() => Int, {nullable:false})
    Format!: number;

    @Field(() => Int, {nullable:false})
    Prompt!: number;

    @Field(() => Int, {nullable:false})
    Help!: number;

    @Field(() => Int, {nullable:false})
    Category!: number;

    @Field(() => Int, {nullable:false})
    Readonly!: number;

    @Field(() => Int, {nullable:false})
    Requires!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
