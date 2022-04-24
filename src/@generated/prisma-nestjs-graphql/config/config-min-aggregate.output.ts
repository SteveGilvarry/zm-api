import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class ConfigMinAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => String, {nullable:true})
    Value?: string;

    @Field(() => String, {nullable:true})
    Type?: string;

    @Field(() => String, {nullable:true})
    DefaultValue?: string;

    @Field(() => String, {nullable:true})
    Hint?: string;

    @Field(() => String, {nullable:true})
    Pattern?: string;

    @Field(() => String, {nullable:true})
    Format?: string;

    @Field(() => String, {nullable:true})
    Prompt?: string;

    @Field(() => String, {nullable:true})
    Help?: string;

    @Field(() => String, {nullable:true})
    Category?: string;

    @Field(() => Int, {nullable:true})
    Readonly?: number;

    @Field(() => String, {nullable:true})
    Requires?: string;
}
