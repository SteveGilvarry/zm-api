import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';

@InputType()
export class GroupsWhereInput {

    @Field(() => [GroupsWhereInput], {nullable:true})
    AND?: Array<GroupsWhereInput>;

    @Field(() => [GroupsWhereInput], {nullable:true})
    OR?: Array<GroupsWhereInput>;

    @Field(() => [GroupsWhereInput], {nullable:true})
    NOT?: Array<GroupsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    ParentId?: IntNullableFilter;
}
