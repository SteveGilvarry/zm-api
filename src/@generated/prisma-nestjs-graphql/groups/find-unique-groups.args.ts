import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';

@ArgsType()
export class FindUniqueGroupsArgs {

    @Field(() => GroupsWhereUniqueInput, {nullable:false})
    where!: GroupsWhereUniqueInput;
}
