import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { IntNullableWithAggregatesFilter } from '../prisma/int-nullable-with-aggregates-filter.input';
import { StringNullableWithAggregatesFilter } from '../prisma/string-nullable-with-aggregates-filter.input';

@InputType()
export class SessionsScalarWhereWithAggregatesInput {

    @Field(() => [SessionsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<SessionsScalarWhereWithAggregatesInput>;

    @Field(() => [SessionsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<SessionsScalarWhereWithAggregatesInput>;

    @Field(() => [SessionsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<SessionsScalarWhereWithAggregatesInput>;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    id?: StringWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    access?: IntNullableWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    data?: StringNullableWithAggregatesFilter;
}
