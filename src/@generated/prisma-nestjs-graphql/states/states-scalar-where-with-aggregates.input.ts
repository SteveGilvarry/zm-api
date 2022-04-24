import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';

@InputType()
export class StatesScalarWhereWithAggregatesInput {

    @Field(() => [StatesScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<StatesScalarWhereWithAggregatesInput>;

    @Field(() => [StatesScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<StatesScalarWhereWithAggregatesInput>;

    @Field(() => [StatesScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<StatesScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Definition?: StringWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    IsActive?: IntWithAggregatesFilter;
}
