import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringNullableWithAggregatesFilter } from '../prisma/string-nullable-with-aggregates-filter.input';
import { IntNullableWithAggregatesFilter } from '../prisma/int-nullable-with-aggregates-filter.input';
import { DateTimeNullableWithAggregatesFilter } from '../prisma/date-time-nullable-with-aggregates-filter.input';

@InputType()
export class SnapshotsScalarWhereWithAggregatesInput {

    @Field(() => [SnapshotsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<SnapshotsScalarWhereWithAggregatesInput>;

    @Field(() => [SnapshotsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<SnapshotsScalarWhereWithAggregatesInput>;

    @Field(() => [SnapshotsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<SnapshotsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    Name?: StringNullableWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    Description?: StringNullableWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    CreatedBy?: IntNullableWithAggregatesFilter;

    @Field(() => DateTimeNullableWithAggregatesFilter, {nullable:true})
    CreatedOn?: DateTimeNullableWithAggregatesFilter;
}
