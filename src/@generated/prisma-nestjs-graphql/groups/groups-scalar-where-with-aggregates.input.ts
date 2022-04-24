import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { IntNullableWithAggregatesFilter } from '../prisma/int-nullable-with-aggregates-filter.input';

@InputType()
export class GroupsScalarWhereWithAggregatesInput {

    @Field(() => [GroupsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<GroupsScalarWhereWithAggregatesInput>;

    @Field(() => [GroupsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<GroupsScalarWhereWithAggregatesInput>;

    @Field(() => [GroupsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<GroupsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    ParentId?: IntNullableWithAggregatesFilter;
}
