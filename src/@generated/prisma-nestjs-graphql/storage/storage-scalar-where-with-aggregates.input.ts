import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { EnumStorage_TypeWithAggregatesFilter } from '../prisma/enum-storage-type-with-aggregates-filter.input';
import { StringNullableWithAggregatesFilter } from '../prisma/string-nullable-with-aggregates-filter.input';
import { BigIntNullableWithAggregatesFilter } from '../prisma/big-int-nullable-with-aggregates-filter.input';
import { EnumStorage_SchemeWithAggregatesFilter } from '../prisma/enum-storage-scheme-with-aggregates-filter.input';
import { IntNullableWithAggregatesFilter } from '../prisma/int-nullable-with-aggregates-filter.input';
import { BoolWithAggregatesFilter } from '../prisma/bool-with-aggregates-filter.input';

@InputType()
export class StorageScalarWhereWithAggregatesInput {

    @Field(() => [StorageScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<StorageScalarWhereWithAggregatesInput>;

    @Field(() => [StorageScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<StorageScalarWhereWithAggregatesInput>;

    @Field(() => [StorageScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<StorageScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Path?: StringWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;

    @Field(() => EnumStorage_TypeWithAggregatesFilter, {nullable:true})
    Type?: EnumStorage_TypeWithAggregatesFilter;

    @Field(() => StringNullableWithAggregatesFilter, {nullable:true})
    Url?: StringNullableWithAggregatesFilter;

    @Field(() => BigIntNullableWithAggregatesFilter, {nullable:true})
    DiskSpace?: BigIntNullableWithAggregatesFilter;

    @Field(() => EnumStorage_SchemeWithAggregatesFilter, {nullable:true})
    Scheme?: EnumStorage_SchemeWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    ServerId?: IntNullableWithAggregatesFilter;

    @Field(() => BoolWithAggregatesFilter, {nullable:true})
    DoDelete?: BoolWithAggregatesFilter;

    @Field(() => BoolWithAggregatesFilter, {nullable:true})
    Enabled?: BoolWithAggregatesFilter;
}
