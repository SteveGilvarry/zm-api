import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';

@ObjectType()
export class Config {

    @Field(() => Int, {nullable:false,defaultValue:0})
    Id!: number;

    @Field(() => ID, {nullable:false})
    Name!: string;

    @Field(() => String, {nullable:false})
    Value!: string;

    @Field(() => String, {nullable:false})
    Type!: string;

    @Field(() => String, {nullable:true})
    DefaultValue!: string | null;

    @Field(() => String, {nullable:true})
    Hint!: string | null;

    @Field(() => String, {nullable:true})
    Pattern!: string | null;

    @Field(() => String, {nullable:true})
    Format!: string | null;

    @Field(() => String, {nullable:true})
    Prompt!: string | null;

    @Field(() => String, {nullable:true})
    Help!: string | null;

    @Field(() => String, {nullable:false})
    Category!: string;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Readonly!: number;

    @Field(() => String, {nullable:true})
    Requires!: string | null;
}
