import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class ConfigMaxOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Value?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultValue?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Hint?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Pattern?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Format?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Prompt?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Help?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Category?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Readonly?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Requires?: keyof typeof SortOrder;
}
