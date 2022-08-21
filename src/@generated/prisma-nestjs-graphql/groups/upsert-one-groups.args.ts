import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';
import { Type } from 'class-transformer';
import { GroupsCreateInput } from './groups-create.input';
import { GroupsUpdateInput } from './groups-update.input';

@ArgsType()
export class UpsertOneGroupsArgs {

    @Field(() => GroupsWhereUniqueInput, {nullable:false})
    @Type(() => GroupsWhereUniqueInput)
    where!: GroupsWhereUniqueInput;

    @Field(() => GroupsCreateInput, {nullable:false})
    @Type(() => GroupsCreateInput)
    create!: GroupsCreateInput;

    @Field(() => GroupsUpdateInput, {nullable:false})
    @Type(() => GroupsUpdateInput)
    update!: GroupsUpdateInput;
}
