import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';

@InputType()
export class MontageLayoutsWhereInput {

    @Field(() => [MontageLayoutsWhereInput], {nullable:true})
    AND?: Array<MontageLayoutsWhereInput>;

    @Field(() => [MontageLayoutsWhereInput], {nullable:true})
    OR?: Array<MontageLayoutsWhereInput>;

    @Field(() => [MontageLayoutsWhereInput], {nullable:true})
    NOT?: Array<MontageLayoutsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Positions?: StringNullableFilter;
}
