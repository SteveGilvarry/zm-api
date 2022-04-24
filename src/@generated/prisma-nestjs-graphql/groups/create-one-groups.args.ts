import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsCreateInput } from './groups-create.input';

@ArgsType()
export class CreateOneGroupsArgs {

    @Field(() => GroupsCreateInput, {nullable:false})
    data!: GroupsCreateInput;
}
