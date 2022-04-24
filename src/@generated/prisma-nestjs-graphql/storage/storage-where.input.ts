import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { EnumStorage_TypeFilter } from '../prisma/enum-storage-type-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';
import { BigIntNullableFilter } from '../prisma/big-int-nullable-filter.input';
import { EnumStorage_SchemeFilter } from '../prisma/enum-storage-scheme-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { BoolFilter } from '../prisma/bool-filter.input';

@InputType()
export class StorageWhereInput {

    @Field(() => [StorageWhereInput], {nullable:true})
    AND?: Array<StorageWhereInput>;

    @Field(() => [StorageWhereInput], {nullable:true})
    OR?: Array<StorageWhereInput>;

    @Field(() => [StorageWhereInput], {nullable:true})
    NOT?: Array<StorageWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Path?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => EnumStorage_TypeFilter, {nullable:true})
    Type?: EnumStorage_TypeFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Url?: StringNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    DiskSpace?: BigIntNullableFilter;

    @Field(() => EnumStorage_SchemeFilter, {nullable:true})
    Scheme?: EnumStorage_SchemeFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    ServerId?: IntNullableFilter;

    @Field(() => BoolFilter, {nullable:true})
    DoDelete?: BoolFilter;

    @Field(() => BoolFilter, {nullable:true})
    Enabled?: BoolFilter;
}
