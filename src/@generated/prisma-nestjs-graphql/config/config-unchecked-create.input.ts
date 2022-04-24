import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class ConfigUncheckedCreateInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:false})
    Name!: string;

    @Field(() => String, {nullable:false})
    Value!: string;

    @Field(() => String, {nullable:false})
    Type!: string;

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

    @Field(() => String, {nullable:false})
    Category!: string;

    @Field(() => Int, {nullable:true})
    Readonly?: number;

    @Field(() => String, {nullable:true})
    Requires?: string;
}
