import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsCreateManyInput } from './groups-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyGroupsArgs {

    @Field(() => [GroupsCreateManyInput], {nullable:false})
    @Type(() => GroupsCreateManyInput)
    data!: Array<GroupsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
