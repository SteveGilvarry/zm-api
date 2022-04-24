import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';
import { GroupsCreateInput } from './groups-create.input';
import { GroupsUpdateInput } from './groups-update.input';

@ArgsType()
export class UpsertOneGroupsArgs {

    @Field(() => GroupsWhereUniqueInput, {nullable:false})
    where!: GroupsWhereUniqueInput;

    @Field(() => GroupsCreateInput, {nullable:false})
    create!: GroupsCreateInput;

    @Field(() => GroupsUpdateInput, {nullable:false})
    update!: GroupsUpdateInput;
}
