import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';

@InputType()
export class ConfigWhereInput {

    @Field(() => [ConfigWhereInput], {nullable:true})
    AND?: Array<ConfigWhereInput>;

    @Field(() => [ConfigWhereInput], {nullable:true})
    OR?: Array<ConfigWhereInput>;

    @Field(() => [ConfigWhereInput], {nullable:true})
    NOT?: Array<ConfigWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    Value?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    Type?: StringFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    DefaultValue?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Hint?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Pattern?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Format?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Prompt?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Help?: StringNullableFilter;

    @Field(() => StringFilter, {nullable:true})
    Category?: StringFilter;

    @Field(() => IntFilter, {nullable:true})
    Readonly?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Requires?: StringNullableFilter;
}
