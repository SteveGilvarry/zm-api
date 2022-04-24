import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';

@InputType()
export class FiltersWhereInput {

    @Field(() => [FiltersWhereInput], {nullable:true})
    AND?: Array<FiltersWhereInput>;

    @Field(() => [FiltersWhereInput], {nullable:true})
    OR?: Array<FiltersWhereInput>;

    @Field(() => [FiltersWhereInput], {nullable:true})
    NOT?: Array<FiltersWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    UserId?: IntNullableFilter;

    @Field(() => StringFilter, {nullable:true})
    Query_json?: StringFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoArchive?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoUnarchive?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoVideo?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoUpload?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoEmail?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    EmailTo?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    EmailSubject?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    EmailBody?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoMessage?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoExecute?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    AutoExecuteCmd?: StringNullableFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoDelete?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoMove?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoCopy?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoCopyTo?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    AutoMoveTo?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    UpdateDiskSpace?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Background?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Concurrent?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    LockRows?: IntFilter;
}
