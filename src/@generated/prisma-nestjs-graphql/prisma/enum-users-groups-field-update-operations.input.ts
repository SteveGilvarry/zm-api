import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Groups } from './users-groups.enum';

@InputType()
export class EnumUsers_GroupsFieldUpdateOperationsInput {

    @Field(() => Users_Groups, {nullable:true})
    set?: keyof typeof Users_Groups;
}
