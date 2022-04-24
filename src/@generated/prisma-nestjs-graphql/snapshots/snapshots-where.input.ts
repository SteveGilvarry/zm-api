import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { DateTimeNullableFilter } from '../prisma/date-time-nullable-filter.input';

@InputType()
export class SnapshotsWhereInput {

    @Field(() => [SnapshotsWhereInput], {nullable:true})
    AND?: Array<SnapshotsWhereInput>;

    @Field(() => [SnapshotsWhereInput], {nullable:true})
    OR?: Array<SnapshotsWhereInput>;

    @Field(() => [SnapshotsWhereInput], {nullable:true})
    NOT?: Array<SnapshotsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Name?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Description?: StringNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    CreatedBy?: IntNullableFilter;

    @Field(() => DateTimeNullableFilter, {nullable:true})
    CreatedOn?: DateTimeNullableFilter;
}
