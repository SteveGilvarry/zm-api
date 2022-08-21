import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsUpdateInput } from './groups-update.input';
import { Type } from 'class-transformer';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';

@ArgsType()
export class UpdateOneGroupsArgs {

    @Field(() => GroupsUpdateInput, {nullable:false})
    @Type(() => GroupsUpdateInput)
    data!: GroupsUpdateInput;

    @Field(() => GroupsWhereUniqueInput, {nullable:false})
    @Type(() => GroupsWhereUniqueInput)
    where!: GroupsWhereUniqueInput;
}
