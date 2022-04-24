import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereInput } from './groups-where.input';

@ArgsType()
export class DeleteManyGroupsArgs {

    @Field(() => GroupsWhereInput, {nullable:true})
    where?: GroupsWhereInput;
}
