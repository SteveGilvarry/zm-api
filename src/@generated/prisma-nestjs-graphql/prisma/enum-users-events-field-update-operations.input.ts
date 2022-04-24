import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Events } from './users-events.enum';

@InputType()
export class EnumUsers_EventsFieldUpdateOperationsInput {

    @Field(() => Users_Events, {nullable:true})
    set?: keyof typeof Users_Events;
}
