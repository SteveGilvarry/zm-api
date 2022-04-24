import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';

@InputType()
export class StatesWhereInput {

    @Field(() => [StatesWhereInput], {nullable:true})
    AND?: Array<StatesWhereInput>;

    @Field(() => [StatesWhereInput], {nullable:true})
    OR?: Array<StatesWhereInput>;

    @Field(() => [StatesWhereInput], {nullable:true})
    NOT?: Array<StatesWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => StringFilter, {nullable:true})
    Definition?: StringFilter;

    @Field(() => IntFilter, {nullable:true})
    IsActive?: IntFilter;
}
