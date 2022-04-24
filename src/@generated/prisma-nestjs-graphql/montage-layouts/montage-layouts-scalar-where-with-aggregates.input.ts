import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { StringNullableWithAggregatesFilter } from '../prisma/string-nullable-with-aggregates-filter.input';

@InputType()
export class MontageLayoutsScalarWhereWithAggregatesInput {

    @Field(() => [MontageLayoutsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<MontageLayoutsScalarWhereWithAggregatesInput>;

    @Field(() => [MontageLayoutsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<MontageLayoutsScalarWhereWithAggregatesInput>;

    @Field(() => [MontageLayoutsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<MontageLayoutsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    Positions?: StringNullableWithAggregatesFilter;
}
