import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';

@InputType()
export class ManufacturersWhereInput {

    @Field(() => [ManufacturersWhereInput], {nullable:true})
    AND?: Array<ManufacturersWhereInput>;

    @Field(() => [ManufacturersWhereInput], {nullable:true})
    OR?: Array<ManufacturersWhereInput>;

    @Field(() => [ManufacturersWhereInput], {nullable:true})
    NOT?: Array<ManufacturersWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;
}
