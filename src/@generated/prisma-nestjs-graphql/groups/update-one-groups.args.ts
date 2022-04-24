import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsUpdateInput } from './groups-update.input';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';

@ArgsType()
export class UpdateOneGroupsArgs {

    @Field(() => GroupsUpdateInput, {nullable:false})
    data!: GroupsUpdateInput;

    @Field(() => GroupsWhereUniqueInput, {nullable:false})
    where!: GroupsWhereUniqueInput;
}
