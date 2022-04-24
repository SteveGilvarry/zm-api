import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Monitors } from './users-monitors.enum';

@InputType()
export class EnumUsers_MonitorsFieldUpdateOperationsInput {

    @Field(() => Users_Monitors, {nullable:true})
    set?: keyof typeof Users_Monitors;
}
