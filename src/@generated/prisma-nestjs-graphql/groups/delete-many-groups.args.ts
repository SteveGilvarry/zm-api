import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsWhereInput } from './groups-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyGroupsArgs {

    @Field(() => GroupsWhereInput, {nullable:true})
    @Type(() => GroupsWhereInput)
    where?: GroupsWhereInput;
}
