import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereUniqueInput } from './groups-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueGroupsArgs {

    @Field(() => GroupsWhereUniqueInput, {nullable:false})
    @Type(() => GroupsWhereUniqueInput)
    where!: GroupsWhereUniqueInput;
}
