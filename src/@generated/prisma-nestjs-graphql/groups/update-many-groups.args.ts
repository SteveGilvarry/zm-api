import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsUpdateManyMutationInput } from './groups-update-many-mutation.input';
import { Type } from 'class-transformer';
import { GroupsWhereInput } from './groups-where.input';

@ArgsType()
export class UpdateManyGroupsArgs {

    @Field(() => GroupsUpdateManyMutationInput, {nullable:false})
    @Type(() => GroupsUpdateManyMutationInput)
    data!: GroupsUpdateManyMutationInput;

    @Field(() => GroupsWhereInput, {nullable:true})
    @Type(() => GroupsWhereInput)
    where?: GroupsWhereInput;
}
