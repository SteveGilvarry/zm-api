import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';

@InputType()
export class ModelsWhereInput {

    @Field(() => [ModelsWhereInput], {nullable:true})
    AND?: Array<ModelsWhereInput>;

    @Field(() => [ModelsWhereInput], {nullable:true})
    OR?: Array<ModelsWhereInput>;

    @Field(() => [ModelsWhereInput], {nullable:true})
    NOT?: Array<ModelsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    ManufacturerId?: IntNullableFilter;
}
