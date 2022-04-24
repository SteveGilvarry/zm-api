import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { NestedDecimalWithAggregatesFilter } from './nested-decimal-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedDecimalFilter } from './nested-decimal-filter.input';

@InputType()
export class DecimalWithAggregatesFilter {

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

    @Field(() => NestedDecimalWithAggregatesFilter, {nullable:true})
    not?: NestedDecimalWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedDecimalFilter, {nullable:true})
    _avg?: NestedDecimalFilter;

    @Field(() => NestedDecimalFilter, {nullable:true})
    _sum?: NestedDecimalFilter;

    @Field(() => NestedDecimalFilter, {nullable:true})
    _min?: NestedDecimalFilter;

    @Field(() => NestedDecimalFilter, {nullable:true})
    _max?: NestedDecimalFilter;
}
