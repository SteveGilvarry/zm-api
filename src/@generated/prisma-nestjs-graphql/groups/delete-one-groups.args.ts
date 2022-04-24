import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';

@ArgsType()
export class DeleteOneGroupsArgs {

    @Field(() => GroupsWhereUniqueInput, {nullable:false})
    where!: GroupsWhereUniqueInput;
}
