import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsCreateInput } from './groups-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneGroupsArgs {

    @Field(() => GroupsCreateInput, {nullable:false})
    @Type(() => GroupsCreateInput)
    data!: GroupsCreateInput;
}
