import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { StringFilter } from '../prisma/string-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';

@InputType()
export class SessionsWhereInput {

    @Field(() => [SessionsWhereInput], {nullable:true})
    AND?: Array<SessionsWhereInput>;

    @Field(() => [SessionsWhereInput], {nullable:true})
    OR?: Array<SessionsWhereInput>;

    @Field(() => [SessionsWhereInput], {nullable:true})
    NOT?: Array<SessionsWhereInput>;

    @Field(() => StringFilter, {nullable:true})
    id?: StringFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    access?: IntNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    data?: StringNullableFilter;
}
