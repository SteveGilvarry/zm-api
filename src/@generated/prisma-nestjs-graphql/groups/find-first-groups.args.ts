import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereInput } from './groups-where.input';
import { GroupsOrderByWithRelationInput } from './groups-order-by-with-relation.input';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';
import { Int } from '@nestjs/graphql';
import { GroupsScalarFieldEnum } from './groups-scalar-field.enum';

@ArgsType()
export class FindFirstGroupsArgs {

    @Field(() => GroupsWhereInput, {nullable:true})
    where?: GroupsWhereInput;

    @Field(() => [GroupsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<GroupsOrderByWithRelationInput>;

    @Field(() => GroupsWhereUniqueInput, {nullable:true})
    cursor?: GroupsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [GroupsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof GroupsScalarFieldEnum>;
}
