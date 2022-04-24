import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { GroupsCreateManyInput } from './groups-create-many.input';

@ArgsType()
export class CreateManyGroupsArgs {

    @Field(() => [GroupsCreateManyInput], {nullable:false})
    data!: Array<GroupsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
