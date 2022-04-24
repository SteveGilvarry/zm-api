import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsUpdateManyMutationInput } from './groups-update-many-mutation.input';
import { GroupsWhereInput } from './groups-where.input';

@ArgsType()
export class UpdateManyGroupsArgs {

    @Field(() => GroupsUpdateManyMutationInput, {nullable:false})
    data!: GroupsUpdateManyMutationInput;

    @Field(() => GroupsWhereInput, {nullable:true})
    where?: GroupsWhereInput;
}
