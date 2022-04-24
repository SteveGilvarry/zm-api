import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringNullableWithAggregatesFilter } from '../prisma/string-nullable-with-aggregates-filter.input';

@InputType()
export class TriggersX10ScalarWhereWithAggregatesInput {

    @Field(() => [TriggersX10ScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<TriggersX10ScalarWhereWithAggregatesInput>;

    @Field(() => [TriggersX10ScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<TriggersX10ScalarWhereWithAggregatesInput>;

    @Field(() => [TriggersX10ScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<TriggersX10ScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    Activation?: StringNullableWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    AlarmInput?: StringNullableWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    AlarmOutput?: StringNullableWithAggregatesFilter;
}
