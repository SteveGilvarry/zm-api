import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';

@InputType()
export class NestedDecimalFilter {

    @Field(() => GraphQLDecimal, {nullable:true})
    equals?: any;

    @Field(() => [GraphQLDecimal], {nullable:true})
    in?: Array<any>;

    @Field(() => [GraphQLDecimal], {nullable:true})
    notIn?: Array<any>;

    @Field(() => GraphQLDecimal, {nullable:true})
    lt?: any;

    @Field(() => GraphQLDecimal, {nullable:true})
    lte?: any;

    @Field(() => GraphQLDecimal, {nullable:true})
    gt?: any;

    @Field(() => GraphQLDecimal, {nullable:true})
    gte?: any;

    @Field(() => NestedDecimalFilter, {nullable:true})
    not?: NestedDecimalFilter;
}
