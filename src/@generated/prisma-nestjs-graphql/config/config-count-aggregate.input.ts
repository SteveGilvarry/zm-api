import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ConfigCountAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Value?: true;

    @Field(() => Boolean, {nullable:true})
    Type?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultValue?: true;

    @Field(() => Boolean, {nullable:true})
    Hint?: true;

    @Field(() => Boolean, {nullable:true})
    Pattern?: true;

    @Field(() => Boolean, {nullable:true})
    Format?: true;

    @Field(() => Boolean, {nullable:true})
    Prompt?: true;

    @Field(() => Boolean, {nullable:true})
    Help?: true;

    @Field(() => Boolean, {nullable:true})
    Category?: true;

    @Field(() => Boolean, {nullable:true})
    Readonly?: true;

    @Field(() => Boolean, {nullable:true})
    Requires?: true;

    @Field(() => Boolean, {nullable:true})
    _all?: true;
}
